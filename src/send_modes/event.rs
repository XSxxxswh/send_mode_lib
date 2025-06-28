use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct Event {
    pub mode_id: String,
    pub bank: String,
    pub amount: Decimal,
    pub requisite: Option<String>,
    pub balance: Option<Decimal>,
    pub search_by: String,
}

#[derive(Debug, Deserialize)]
pub struct SendEvent {
    pub source: String,
    pub text: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TextMessage {
    pub mode_id: String,
    pub source: String,
    pub text: String,
}

#[derive(Debug, Serialize)]
pub struct Context {
    pub amount: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub balance: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requisite: Option<String>,
}