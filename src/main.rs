use chrono::prelude::*;
use hyper::body::to_bytes;
use hyper::{Body, Client, Method, Request};
use hyper_tls::HttpsConnector;
use scraper::{Html, Selector};
use serde::Deserialize;
use serde_urlencoded;
use serde_yaml;
use std;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::thread;
use std::time::Duration;

#[derive(Debug, Deserialize)]
struct Config {
    base_url: String,
    timeslot: String,
    registrants: Vec<Registrant>,
}

#[derive(Debug, Deserialize)]
struct Registrant {
    name: String,
    surname: String,
    email_address: String,
}

fn load_file(file_path: &str) -> String {
    let yaml_content = match fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error reading file: {}", err);
            std::process::exit(1);
        }
    };
    yaml_content
}

fn read_config(yaml_content: &str) -> Config {
    let config: Config = match serde_yaml::from_str(yaml_content) {
        Ok(config) => config,
        Err(err) => {
            eprintln!("Error parsing YAML: {}", err);
            std::process::exit(1);
        }
    };
    config
}

#[allow(dead_code)]
fn replace_timeslot(config: &Config) -> String {
    let url = &config.base_url;
    url.replace("timeslot", &config.timeslot)
}

#[allow(dead_code)]
fn next_weekday(date: NaiveDate, target_weekday: Weekday) -> String {
    let mut current_date = date;
    while current_date.weekday() != target_weekday {
        current_date = current_date.succ_opt().expect("Already at max date!");
    }
    current_date.format("%Y%m%d").to_string()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: cargo run -- <file_path>");
        std::process::exit(1);
    }

    let file_path = &args[1];

    let yaml_content = load_file(file_path);

    let config: Config = read_config(&yaml_content);

    // let mut updated_url = replace_timeslot(&config) + "-";

    // let today = Local::now().date_naive();

    // let next_thursday = next_weekday(today, Weekday::Thu);

    // updated_url.push_str(&next_thursday);

    let updated_url = config.base_url;

    let https = HttpsConnector::new();

    let client = Client::builder().build::<_, hyper::Body>(https);

    let request = Request::builder()
        .method(Method::POST)
        .uri(&updated_url)
        .body(Body::from(""))?;

    let response = client.request(request).await?;

    let response_body = to_bytes(response.into_body()).await?;

    let body = String::from_utf8(response_body.to_vec())?;

    let document = Html::parse_document(&body);

    let selector_for_404 =
        Selector::parse("#page-title").expect("Couldn't parse page to check if it was a 404.");

    for element in document.select(&selector_for_404) {
        if element.text().collect::<String>() == "404: Page not found" {
            eprintln!("Got a 404 error for URL '{}'!", &updated_url);
            std::process::exit(1);
        }
    }

    let selectors: Vec<Selector> = (1..=3)
        .map(|i| {
            Selector::parse(&format!(
                "#registration-form > div > input[type=hidden]:nth-child({})",
                i
            ))
            .expect(&format!(
                "Couldn't parse selector nth-child element for '{}'!",
                i
            ))
        })
        .collect();

    let mut attributes: Vec<&str> = Vec::new();

    for (i, selector) in selectors.into_iter().enumerate() {
        let element = document.select(&selector).nth(0);
        attributes.push(
            element
                .expect(&format!("Couldn't get value for element '{}'.", i))
                .value()
                .attr("value")
                .expect(&format!(
                    "Couldn't get attribute 'value' for element '{}'.",
                    i
                )),
        );
    }

    let mut data = HashMap::new();

    data.insert("op", "RSVP");
    data.insert("form_build_id", attributes[0]);
    data.insert("form_id", attributes[1]);
    data.insert("honeypot_time", attributes[2]);

    for registrant in &config.registrants {
        data.insert("field_registration_name[und][0][value]", &registrant.name);
        data.insert(
            "field_registration_lname[und][0][value]",
            &registrant.surname,
        );
        data.insert("anon_mail", &registrant.email_address);

        let data_string = serde_urlencoded::to_string(&data)?;

        let register_request = Request::builder()
            .method(Method::POST)
            .uri(&updated_url)
            .header("content-type", "application/x-www-form-urlencoded")
            // .header("content-type", "application/json")
            // .header("authority", "www.bklynlibrary.org")
            // .header("accept", "text/html,application/xhtml+xml,application/xmlq=0.9,image/avif,image/webp,image/apng,*/*q=0.8,application/signed-exchangev=b3q=0.7")
            // .header(
            //     "accept-language",
            //     "en-US,enq=0.9,plq=0.8",
            // )
            // .header("cache-control", "no-cache")
            // .header("dnt", "1")
            // .header("pragma", "no-cache")
            // .header(
            //     "referer",
            //     "https: //www.bklynlibrary.org/locations/brooklyneights",
            // )
            // .header(
            //     "sec-ch-ua",
            //     "\"Google Chrome\"v=\"113\", \"Chromium\"v=\"113\", \"Not-A.Brand\"v=\"24\"",
            // )
            // .header("sec-ch-ua-mobile", "?0")
            // .header("sec-ch-ua-platform", "\"Windows\"")
            // .header("sec-fetch-dest", "document")
            // .header("sec-fetch-mode", "navigate")
            // .header("sec-fetch-site", "same-origin")
            // .header("sec-fetch-user", "?1")
            // .header("upgrade-insecure-requests", "1")
            // .header("user-agent", "Mozilla/5.0 (Windows NT 10.0 Win64 x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/113.0.0.0 Safari/537.36")
            // .header(
            //     "Cookie",
            //     "Drupal.tableDrag.showWeight=0",
            // )
            .body(hyper::Body::from(data_string))?;

        println!("{:#?}", register_request);

        thread::sleep(Duration::from_secs(5));

        let register_response = client.request(register_request).await?;

        println!("Response: {:#?}", register_response.headers());
    }

    Ok(())
}
