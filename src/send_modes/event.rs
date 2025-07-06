use std::fmt::Display;
use std::str::FromStr;
use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Eq, PartialEq)]
pub enum EventType {
    SMS,
    PUSH
}

impl From<&str> for EventType {
    fn from(s: &str) -> Self {
        EventType::from_str(s).unwrap()
    }
}

impl FromStr for EventType {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "sms" => Ok(EventType::SMS),
            "push_notification" => Ok(EventType::PUSH),
            _ => Err(()),
        }
    }
}

impl Display for EventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventType::SMS => write!(f, "sms"),
            EventType::PUSH => write!(f, "push_notification"),
        }
    }
}

impl From<EventType> for String {
    fn from(event: EventType) -> Self {
        event.to_string()
    }
}

impl From<&EventType> for String {
    fn from(event: &EventType) -> Self {
        event.to_string()
    }
}

impl From<String> for EventType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "sms" => EventType::SMS,
            "push_notification" => EventType::PUSH,
            _ => unreachable!()
        }
    }
}

impl Serialize for EventType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s: String = self.into();
        serializer.serialize_str(&s)
    }
}

impl<'de> Deserialize<'de> for EventType {
    fn deserialize<D>(deserializer: D) -> Result<EventType, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(EventType::from(s))
    }
}

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
    pub event_type: EventType,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TextMessage {
    pub mode_id: String,
    pub source: String,
    pub text: String,
    pub event_type: EventType,
}

#[derive(Debug, Serialize)]
pub struct Context {
    pub amount: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub balance: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requisite: Option<String>,
}