pub mod send_mode_client;

use std::time::Duration;
use tokio_retry2::{Retry, RetryError};
use tokio_retry2::strategy::{ExponentialBackoff, FixedInterval};
use tracing::{debug, error, warn};
use crate::send_modes::error::LibError;

async fn send_request_for_retry(client: &reqwest::Client, request: reqwest::Request)
                                -> Result<reqwest::Response, RetryError<LibError>>
{
    let url = request.url().to_string();
    match tokio::time::timeout(Duration::from_millis(500), client.execute(request)).await {
        Ok(Ok(response)) => Ok(response),
        Ok(Err(e)) => {
            warn!(err=e.to_string(), url=url, "Request send error. Retrying...");
            return Err(RetryError::to_transient(LibError::IOError(e))?) // TODO: тут возможна ошибка
        }
        Err(_) => {
            warn!(url=url, "Request send timed out. Retrying...");
            return Err(RetryError::to_transient(LibError::TimeOut)?)
        }
    }
}

pub async fn send_request(client: &reqwest::Client, request: reqwest::Request)
                          -> Result<reqwest::Response, LibError>
{
    let request = match request.try_clone() {
        Some(req) => req,
        None => {
            error!("error clone request");
            return Err(LibError::InternalServerError)?
        }
    };
    let start = tokio::time::Instant::now();
    let retry_strategy = FixedInterval::from_millis(100)
        .take(5);
    let resp = Retry::spawn(retry_strategy, move || {
        let client = client.clone();
        let request = request.try_clone().ok_or(RetryError::Permanent(LibError::InternalServerError));

        async move {
            let req = request?;
            send_request_for_retry(&client, req).await
        }
    }).await;
    let elapsed = start.elapsed();
    debug!("request elapsed in {}", elapsed.as_millis());
    resp
}



#[macro_export]
macro_rules! retry {
    ($sql_func:expr, $max_retries:expr) => {{
        let mut result;
        let mut attempt = 0;
        loop {
            attempt += 1;
            result = tokio::time::timeout(Duration::from_millis(5000), $sql_func).await;
            match result {
                Ok(Ok(r)) => {
                    break Ok(r)
                },
                Ok(Err(ref e)) if attempt < $max_retries && is_connection_err(e) => {
                    warn!(err=e.to_string(), "Error do request. Retrying...");
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    continue;
                },
                Ok(Err(e)) => break Err(e.to_string()),
                Err(e)if attempt < $max_retries => {
                    warn!(err=e.to_string(), "timeout do request. Retrying...");
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    continue;
                }
                Err(e) => {
                    break Err(e.to_string());
                }
            }
        }
    }};
}
pub fn is_connection_err<T>(e: &T) -> bool
where T: ToString
{
    e.to_string().contains("connection")
        || e.to_string().contains("broken")
        || e.to_string().contains("time")
        || e.to_string().contains("timed")
        || e.to_string().contains("conn")
}