use std::error::Error;
use std::fmt::Display;
use std::str::FromStr;
use chrono::{DateTime, Utc};
use deadpool_postgres::tokio_postgres;
use deadpool_postgres::tokio_postgres::types::{FromSql, IsNull, ToSql, Type};
use deadpool_postgres::tokio_postgres::types::private::BytesMut;
use deadpool_redis::redis;
use deadpool_redis::redis::{FromRedisValue, RedisError, RedisResult, RedisWrite, ToRedisArgs, Value};
use rsa::pkcs1::{DecodeRsaPrivateKey, EncodeRsaPrivateKey};
use rsa::RsaPrivateKey;
use serde::{Deserialize, Serialize, Serializer};
use tracing::{error};
use bytes::buf::BufMut;
use reqwest::Body;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum SendModeEnum {
    KRAFT,
    TRADEMO
}

impl ToSql for SendModeEnum {
    fn to_sql(&self, _ty: &Type, out: &mut BytesMut) -> Result<IsNull, Box<dyn Error + Sync + Send>>
    where
        Self: Sized
    {
        match self {
            SendModeEnum::KRAFT => out.put(&b"KRAFT"[..]),

            SendModeEnum::TRADEMO => out.put(&b"TRADEMO"[..]),
        }
        Ok(IsNull::No)
    }

    fn accepts(ty: &Type) -> bool
    where
        Self: Sized
    {
        matches!(*ty, Type::VARCHAR | Type::TEXT)
    }

    fn to_sql_checked(&self, _ty: &Type, out: &mut BytesMut) -> Result<IsNull, Box<dyn Error + Sync + Send>> {
        match self {
            SendModeEnum::KRAFT => out.put(&b"KRAFT"[..]),

            SendModeEnum::TRADEMO => out.put(&b"TRADEMO"[..]),
        }
        Ok(IsNull::No)
    }
}

impl Display for SendModeEnum {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SendModeEnum::KRAFT => write!(f, "KRAFT"),
            SendModeEnum::TRADEMO => write!(f, "TRADEMO")
        }
    }
}
impl FromStr for SendModeEnum {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "KRAFT" => Ok(SendModeEnum::KRAFT),
            "TRADEMO" => Ok(SendModeEnum::TRADEMO),
            _ => Err(())
        }
    }
}

impl FromSql<'_> for SendModeEnum {
    fn from_sql<'a>(_ty: &Type, raw: &'a [u8]) -> Result<Self, Box<dyn Error + Sync + Send>> {
        match raw {
            b"KRAFT" => Ok(SendModeEnum::KRAFT),
            b"TRADEMO" => Ok(SendModeEnum::TRADEMO),
            _ => Err("Invalid parameter".into())
        }
    }

    fn accepts(ty: &Type) -> bool {
        matches!(*ty, Type::VARCHAR | Type::TEXT)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMode {
    pub id: String,
    pub aggregate_id: String,
    pub name: String,
    pub send_mode: SendModeEnum,
    pub access_token: String,
    pub fingerprint: Option<String>,
    #[serde(serialize_with = "rsa_serialize")]
    #[serde(deserialize_with = "rsa_deserialize")]
    pub private_key: Option<RsaPrivateKey>,
    pub auto_heartbeat_interval: Option<i32>,
    pub last_heartbeat: DateTime<Utc>,
}


pub fn rsa_serialize<S>(key: &Option<RsaPrivateKey>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match key {
        Some(k) => {
            let pem = k.to_pkcs1_pem(rsa::pkcs1::LineEnding::LF)
                .map_err(serde::ser::Error::custom)?;
            serializer.serialize_str(&pem)
        }
        None => serializer.serialize_none(),
    }
}

pub fn rsa_deserialize<'de, D>(deserializer: D) -> Result<Option<RsaPrivateKey>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    // Сначала десериализуем Option<String>
    let opt = Option::<String>::deserialize(deserializer)?;
    if let Some(pem_str) = opt {
        // Пытаемся распарсить PEM строку
        RsaPrivateKey::from_pkcs1_pem(&pem_str)
            .map(Some)
            .map_err(serde::de::Error::custom)
    } else {
        Ok(None)
    }
}

impl From<tokio_postgres::row::Row> for SendMode {
    fn from(row: tokio_postgres::Row) -> Self {
        Self {
            id: row.get("id"),
            aggregate_id: row.get("aggregate_id"),
            name: row.get("name"),
            send_mode: row.get("send_mode"),
            access_token: row.get("access_token"),
            fingerprint: row.get("fingerprint"),
            private_key: RsaPrivateKey::from_pkcs1_pem(row.get("private_key")).ok(),
            auto_heartbeat_interval: row.get("auto_heartbeat_interval"),
            last_heartbeat: row.get("last_heartbeat"),
        }
    }
}

impl From<&tokio_postgres::row::Row> for SendMode {
    fn from(row: &tokio_postgres::Row) -> Self {
        Self {
            id: row.get("id"),
            aggregate_id: row.get("aggregate_id"),
            name: row.get("name"),
            send_mode: row.get("send_mode"),
            access_token: row.get("access_token"),
            fingerprint: row.get("fingerprint"),
            private_key: RsaPrivateKey::from_pkcs1_pem(row.get("private_key")).ok(),
            auto_heartbeat_interval: row.get("auto_heartbeat_interval"),
            last_heartbeat: row.get("last_heartbeat"),
        }
    }
}


impl ToRedisArgs for SendMode {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + RedisWrite
    {
        out.write_arg(simd_json::to_vec(&self).unwrap().as_slice());
    }
}

impl FromRedisValue for SendMode {
    fn from_redis_value(v: &Value) -> RedisResult<Self> {
        match v {

            Value::BulkString(f) => {
                Ok(serde_json::from_slice(f).map_err(|e| {
                    error!(err=e.to_string(), "Error deserialize payload");
                    RedisError::from((redis::ErrorKind::TypeError, "Cannot deserialize send_mode"))
                })?)
            },
            _ => Err(RedisError::from((redis::ErrorKind::TypeError, "Cannot deserialize send_mode unknown format")))
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewSendModeRequest {
    pub aggregate_id: String,
    pub name: String,
    pub mode: SendModeEnum,
    pub access_token: String,
    pub auto_heartbeat_interval: Option<i32>,
}

impl Into<Body> for NewSendModeRequest {
    fn into(self) -> Body {
        reqwest::Body::from(simd_json::to_vec(&self).unwrap())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenameSendModeRequest {
    pub id: String,
    pub name: String
}