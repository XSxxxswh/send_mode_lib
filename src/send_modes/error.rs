use thiserror::Error;

#[derive(Debug, Error)]
pub enum LibError {
    #[error("InternalError")]
    InternalServerError,
    #[error("request timeout")]
    TimeOut,
    #[error("IOError")]
    IOError(#[from] reqwest::Error),
    #[error("{0}, Not found")]
    NotFound(String),
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Forbidden")]
    Forbidden,
    #[error("Invalid device mode")]
    InvalidDeviceMode
}



