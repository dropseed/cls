use crate::events::Event;
use reqwest;
use reqwest::header::{self, HeaderMap, HeaderValue};
use reqwest::Error as ReqwestError;
use serde_json;
use std::env;

const DEFAULT_API_URL: &str = "https://api.cls.dev/";

pub struct APIClient {
    base_url: String,
    client: reqwest::blocking::Client,
}

impl APIClient {
    pub fn new(token: &str) -> APIClient {
        let mut base_url = match env::var("CLS_API_URL") {
            Ok(url) => url,
            Err(_) => DEFAULT_API_URL.to_string(),
        };

        // Make sure it ends with a slash
        if !base_url.to_string().ends_with("/") {
            base_url += "/";
        }

        let mut auth_value = HeaderValue::from_str(format!("Token {}", token).as_str()).unwrap();
        let mut headers = HeaderMap::new();

        auth_value.set_sensitive(true);
        headers.insert(header::AUTHORIZATION, auth_value);
        headers.insert(
            header::ACCEPT,
            HeaderValue::from_str("application/json").unwrap(),
        );
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_str("application/json").unwrap(),
        );
        headers.insert(
            header::USER_AGENT,
            HeaderValue::from_str("cls-api-client").unwrap(),
        );

        // get a client builder
        let client = reqwest::blocking::Client::builder()
            .default_headers(headers)
            .build()
            .unwrap();

        APIClient {
            base_url: base_url,
            client: client,
        }
    }
    fn post(&self, path: &str, json: &serde_json::Value) -> Result<(), ReqwestError> {
        let mut url = self.base_url.to_string() + path.strip_prefix("/").unwrap_or(path);

        // Make sure it ends with a slash
        if !url.to_string().ends_with("/") {
            url += "/";
        }

        let res = self.client.post(url).json(&json).send()?;

        super::debug_print(format!(
            "api_post {} {}",
            res.status(),
            res.text().unwrap_or("<no text>".to_string())
        ));

        Ok(())
    }
    pub fn post_event(&self, event: &Event) -> Result<(), ReqwestError> {
        let json = serde_json::json!({
            "slug": event.slug,
            "type": event.type_s,
            "metadata": event.metadata,
            "datetime": event.datetime,
            "user_id": event.user_id,
            "invocation_id": event.invocation_id,
        });
        self.post("events/", &json)
    }
}
