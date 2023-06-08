use hyper::body::to_bytes;
use hyper::header::LOCATION;
use hyper::{Body, Client, Method, Request, Response};
use hyper_tls::HttpsConnector;
use scraper::{Html, Selector};
use std;
use std::time::Duration;

mod errors;
mod models;

use errors::PageParsingError;
use models::{FormAttributes, Registrant, RegistrationRequest};

async fn get_response_to_html(
    response: Response<Body>,
) -> Result<Html, Box<dyn std::error::Error + Send + Sync>> {
    let response_body_bytes = to_bytes(response.into_body()).await?;

    let response_body_str = String::from_utf8_lossy(&response_body_bytes).into_owned();

    Ok(Html::parse_document(&response_body_str))
}

// Perform a POST request to get the page content for a URI, because we want some
// attributes to appear in the page response that woulnd't be present with a GET,
// such as `honeypot_time`.
async fn get_response(
    client: &Client<HttpsConnector<hyper::client::HttpConnector>>,
    uri: &str,
) -> Result<Response<Body>, Box<dyn std::error::Error + Send + Sync>> {
    let request = Request::builder()
        .method(Method::POST)
        .uri(uri)
        .body(Body::from(""))?;

    Ok(client.request(request).await?)
}

fn get_page_title() -> Selector {
    Selector::parse("#page-title").unwrap()
}

async fn check_status_404(
    document: &Html,
    title_selector: &Selector,
) -> Result<(), PageParsingError> {
    match document.select(title_selector).next() {
        None => Ok(()),
        Some(element) => match element.text().collect::<String>().as_str() {
            "404: Page not found" => Err(PageParsingError::PageNotFoundError),
            _ => Ok(()),
        },
    }
}

async fn check_registration_open(document: &Html) -> Result<(), PageParsingError> {
    let registration_closed_selector = Selector::parse("#regMessage > div").unwrap();

    match document.select(&registration_closed_selector).next() {
        None => Ok(()),
        Some(element) => match element.text().collect::<String>().as_str() {
            will_open if will_open.contains("Registration will open on") => {
                Err(PageParsingError::NotSuccessfulPageError)
            }
            "Registration has been closed.." => Err(PageParsingError::NotSuccessfulPageError),
            _ => Ok(()),
        },
    }
}

fn get_registration_form_selectors() -> Vec<Selector> {
    (1..=3)
        .map(|i| {
            Selector::parse(&format!(
                "#registration-form > div > input[type=hidden]:nth-child({})",
                i
            ))
            .unwrap()
        })
        .collect()
}

async fn get_registration_form_attributes(
    document: &Html,
    selectors: Vec<Selector>,
) -> FormAttributes {
    let mut attributes: Vec<String> = Vec::new();

    for selector in selectors {
        let element: scraper::ElementRef = document.select(&selector).next().unwrap();
        attributes.push(element.value().attr("value").unwrap().to_string());
    }

    FormAttributes::from_iter(attributes)
}

async fn build_post_data(registrant: Registrant, attributes: FormAttributes) -> hyper::Body {
    let registration_request_params = RegistrationRequest {
        name: registrant.name,
        surname: registrant.surname,
        email_address: registrant.email_address,
        form_build_id: attributes.form_build_id,
        form_id: attributes.form_id,
        honeypot_time: attributes.honeypot_time,
    };

    hyper::Body::from(registration_request_params.to_string().unwrap())
}

async fn post_registration_request(
    client: &Client<HttpsConnector<hyper::client::HttpConnector>>,
    uri: &str,
    request_body: hyper::Body,
) -> Result<Response<Body>, Box<dyn std::error::Error + Send + Sync>> {
    let request = Request::builder()
        .method(Method::POST)
        .uri(uri)
        .header("content-type", "application/x-www-form-urlencoded")
        .body(request_body)?;

    // Need to wait as `honeypot_time` will prevent us from successfully
    // posting immediately.
    tokio::time::sleep(Duration::from_secs(5)).await;

    Ok(client.request(request).await?)
}

async fn check_status_confirmation(response: &Response<Body>) -> Result<(), PageParsingError> {
    match response.headers().get(LOCATION) {
        None => Err(PageParsingError::NotSuccessfulPageError),
        Some(uri) => match uri.to_str().unwrap() {
            suffix if suffix.ends_with("event-rsvp") => Ok(()),
            _ => Err(PageParsingError::NotSuccessfulPageError),
        },
    }
}

#[tokio::main]
pub async fn register(
    uri: String,
    name: String,
    surname: String,
    email_address: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let registrant = Registrant {
        name: name,
        surname: surname,
        email_address: email_address,
    };

    let https = HttpsConnector::new();

    let client = Client::builder().build::<_, hyper::Body>(https);

    let response = get_response(&client, &uri).await?;

    let document = get_response_to_html(response).await?;

    let title_selector = get_page_title();

    check_status_404(&document, &title_selector).await?;

    check_registration_open(&document).await?;

    let selectors = get_registration_form_selectors();

    let form_attributes = get_registration_form_attributes(&document, selectors).await;

    let post_request_body = build_post_data(registrant, form_attributes).await;

    let confirmation_response = post_registration_request(&client, &uri, post_request_body).await?;

    check_status_confirmation(&confirmation_response).await?;

    Ok(())
}
