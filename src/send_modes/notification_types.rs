use deadpool_postgres::tokio_postgres;
use deadpool_redis::redis;
use deadpool_redis::redis::{ErrorKind, FromRedisValue, RedisError, RedisWrite, ToRedisArgs, Value};
use serde::{Deserialize, Serialize};
use tracing::error;
use crate::send_modes::send_mode::SendModeEnum;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct NotificationTemplate {
    pub bank: String,
    pub send_mode: SendModeEnum,
    pub template: String,
    pub search_by: String,
    pub has_requisite: bool,
    pub has_balance: bool,
    pub notification_type: String,
    pub source: String,
    pub need_to_replace_comma: bool,
}

impl From<&tokio_postgres::Row> for NotificationTemplate {
    fn from(row: &tokio_postgres::Row) -> Self {
        Self {
            bank: row.get("bank"),
            send_mode: row.get("send_mode"),
            template: row.get("template"),
            search_by: row.get("search_by"),
            has_requisite: row.get("has_requisite"),
            has_balance: row.get("has_balance"),
            notification_type: row.get("notification_type"),
            source: row.get("source"),
            need_to_replace_comma: row.get("need_to_replace_comma"),
        }
    }
}

impl From<tokio_postgres::Row> for NotificationTemplate {
    fn from(row: tokio_postgres::Row) -> Self {
        Self {
            bank: row.get("bank"),
            send_mode: row.get("send_mode"),
            template: row.get("template"),
            search_by: row.get("search_by"),
            has_requisite: row.get("has_requisite"),
            has_balance: row.get("has_balance"),
            notification_type: row.get("notification_type"),
            source: row.get("source"),
            need_to_replace_comma: row.get("need_to_replace_comma"),
        }
    }
}


impl FromRedisValue for NotificationTemplate {
    fn from_redis_value(v: &redis::Value) -> redis::RedisResult<Self> {
        match v {
            Value::BulkString(b) => {
                let template = serde_json::from_slice(b.as_slice()).map_err(|e| {
                    error!(err=e.to_string(), "Error deserialize value from redis");
                    RedisError::from((ErrorKind::TypeError, "Value type error"))
                })?;
                Ok(template)
            }
            _s => Err(RedisError::from((ErrorKind::TypeError, "Invalid type for notification template"))),
        }
    }
}

impl ToRedisArgs for NotificationTemplate {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + RedisWrite
    {
        out.write_arg(simd_json::to_vec(self).map_err(|e| {
            error!(err=e.to_string(),"Error serializing notification template");
        }).unwrap().as_slice());
    }
}