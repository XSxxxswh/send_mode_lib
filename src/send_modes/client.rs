use std::env;
use std::time::Duration;
use once_cell::sync::Lazy;
use tokio_retry2::{Retry, RetryError};
use tokio_retry2::strategy::ExponentialBackoff;
use tracing::{debug, error, warn};
use crate::send_modes::send_mode::{NewSendModeRequest, SendMode};
use crate::tools;
use bytes::BytesMut;
const SEND_MODE_API_URL: Lazy<String> = Lazy::new(|| {
    env::var("SEND_MODE_API_URL").expect("Set SEND_MODE_API_URL environment variable")
});

pub struct ClientSendModes {
    client: reqwest::Client,
}

impl ClientSendModes {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
    pub fn new_with_client(client: reqwest::Client) -> Self {
        Self { client }
    }

    pub async fn new_send_mode(&self, request: NewSendModeRequest) -> Result<SendMode, Box<dyn std::error::Error>> {
        let payload = simd_json::to_vec(&request).map_err(|err| err.to_string())?;
        let request = self.client.post(SEND_MODE_API_URL.as_str().to_owned() + "api/v1/send_modes")
        .body(payload).header("Content-Type", "application/json").build()?;
        let response = tools::send_request(&self.client, request).await?;
        match response.error_for_status() {
            Ok(res) => {
                let mut body = res.bytes().await?;
                Ok(serde_json::from_slice(body.as_ref()).unwrap())
            }
            Err(err) => {

                error!(code=?err.status(),"New send mode creation error {}", err.to_string());
                Err(err.into())
            }
        }
    }
}