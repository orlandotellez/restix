use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisData {
    pub keys: Vec<KeyInfo>,
    pub total_memory: u64,
    pub connected: bool,
    pub error: Option<String>,
}

impl Default for RedisData {
    fn default() -> Self {
        Self {
            keys: Vec::new(),
            total_memory: 0,
            connected: false,
            error: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyInfo {
    pub name: String,
    pub key_type: String,
    pub ttl: i64,
    pub memory_bytes: u64,
    pub value_preview: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TtlStatus {
    NoExpiration,
    Expired,
    Urgent(i64),
    Warning(i64),
    Normal(i64),
}

impl TtlStatus {
    pub fn from_ttl(ttl: i64) -> Self {
        match ttl {
            -1 => TtlStatus::NoExpiration,
            ttl if ttl < 60 => TtlStatus::Urgent(ttl),
            ttl if ttl < 3600 => TtlStatus::Warning(ttl),
            _ => TtlStatus::Normal(ttl),
        }
    }
}
