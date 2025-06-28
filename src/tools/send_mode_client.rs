use std::env;
use once_cell::sync::Lazy;
use reqwest::Client;
use tracing::error;
use crate::send_modes;
use crate::send_modes::error::LibError;
use crate::send_modes::error::LibError::{IOError, InternalServerError};
use crate::send_modes::send_mode::{NewSendModeRequest, SendMode};
use crate::tools::send_request;

const SEND_MODE_URL: Lazy<String> = Lazy::new(|| {
    env::var("SEND_MODE_URL").expect("Set SEND_MODE_URL environment variable")
});

pub struct SendModeClient {
    client: Client
}
impl SendModeClient {
    pub fn new() -> Self {
        Self {
            client: Client::new()
        }
    }
    pub async fn new_send_mode(&self, request: NewSendModeRequest)
    -> Result<SendMode, LibError>
    {
        let request = self.client.post(format!("{}/api/v1/send_modes", *SEND_MODE_URL).as_str())
            .body(request).build()?;
        let response = send_request(&self.client, request).await?;
        match response.error_for_status() {
            Ok(response) => {
                let payload = response.text().await?;
                Ok(serde_json::from_str::<SendMode>(&payload).map_err(|e| {
                    error!(err=e.to_string(), "body serialize error");
                    InternalServerError
                })?)
            },
            Err(e) => {
                error!(err=e.to_string(), "send mode error");
                Err(InternalServerError)
            }
        }
    }
    pub async fn get_send_mode_by_id(&self, send_mode_id: &str)
                               -> Result<SendMode, LibError>
    {
        let request = self.client.get(format!("{}/api/v1/send_modes/{}", *SEND_MODE_URL, send_mode_id).as_str()).build()?;
        let response = send_request(&self.client, request).await?;
        match response.error_for_status() {
            Ok(response) => {
                let payload = response.text().await?;
                Ok(serde_json::from_str::<SendMode>(&payload).map_err(|e| {
                    error!(err=e.to_string(), "body serialize error");
                    InternalServerError
                })?)
            },
            Err(e) => {
                error!(err=e.to_string(), "send mode error");
                Err(InternalServerError)
            }
        }
    }
    pub async fn heartbeat(&self, send_mode_id: &str)
                                     -> Result<(), LibError>
    {
        let request = self.client.get(format!("{}/api/v1/send_modes/{}/heartbeat", *SEND_MODE_URL, send_mode_id).as_str()).build()?;
        let response = send_request(&self.client, request).await?;
        match response.error_for_status() {
            Ok(response) => {
                return Ok(())
            },
            Err(e) => {
                error!(err=e.to_string(), "send mode error");
                Err(InternalServerError)
            }
        }
    }
    pub async fn get_send_mode_by_aggregate_id(&self, send_mode_id: &str)
                                     -> Result<Vec<SendMode>, LibError>
    {
        let request = self.client.get(format!("{}/api/v1/send_modes/aggregate_id/{}", *SEND_MODE_URL, send_mode_id).as_str()).build()?;
        let response = send_request(&self.client, request).await?;
        match response.error_for_status() {
            Ok(response) => {
                let payload = response.text().await?;
                Ok(serde_json::from_str::<Vec<SendMode>>(&payload).map_err(|e| {
                    error!(err=e.to_string(), "body serialize error");
                    InternalServerError
                })?)
            },
            Err(e) => {
                error!(err=e.to_string(), "send mode error");
                Err(InternalServerError)
            }
        }
    }
    pub async fn delete_send_mode(&self, send_mode_id: &str) -> Result<(), LibError>
    {
        let request = self.client.delete(format!("{}/api/v1/send_modes/{}", *SEND_MODE_URL, send_mode_id).as_str()).build()?;
        let response = send_request(&self.client, request).await?;
        match response.error_for_status() {
            Ok(_) => {
                Ok(())
            },
            Err(e) => {
                error!(err=e.to_string(), "send mode error");
                Err(InternalServerError)
            }
        }
    }
}