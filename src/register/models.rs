use serde::Serialize;
use serde_urlencoded;

#[derive(Debug)]
pub struct Registrant {
    pub name: String,
    pub surname: String,
    pub email_address: String,
}

#[derive(Debug)]
pub struct FormAttributes {
    pub form_build_id: String,
    pub form_id: String,
    pub honeypot_time: String,
}

impl FromIterator<String> for FormAttributes {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = String>,
    {
        let mut iter = iter.into_iter();
        Self {
            form_build_id: iter.next().unwrap(),
            form_id: iter.next().unwrap(),
            honeypot_time: iter.next().unwrap(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct RegistrationRequest {
    #[serde(rename = "field_registration_name[und][0][value]")]
    pub name: String,
    #[serde(rename = "field_registration_lname[und][0][value]")]
    pub surname: String,
    #[serde(rename = "anon_mail")]
    pub email_address: String,
    pub form_build_id: String,
    pub form_id: String,
    pub honeypot_time: String,
}

impl RegistrationRequest {
    pub fn to_string(&self) -> Result<String, serde_urlencoded::ser::Error> {
        serde_urlencoded::to_string(self)
    }
}
