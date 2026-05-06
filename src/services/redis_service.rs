use crate::{
    models::redis_model::{KeyInfo, RedisData},
    utils::formatting::try_format_as_json,
};
use anyhow::{Context, Result};
use redis::{Client, Commands, Connection};

pub struct RedisService {
    client: Client,
    connection: Option<Connection>,
    pub total_memory: u64,
    pub connected: bool,     // Track connection state
    pub current_url: String, // Track current URL
}

impl RedisService {
    pub fn new(url: &str) -> Result<Self> {
        let client = Client::open(url).context("Failed to create Redis client")?;
        Ok(Self {
            client,
            connection: None,
            total_memory: 0,
            connected: false,
            current_url: url.to_string(),
        })
    }

    pub fn connect(&mut self) -> Result<bool> {
        // Use same logic as reconnect - copy URL first to avoid borrow issues
        let url = self.current_url.clone();
        match Client::open(url) {
            Ok(client) => {
                match client.get_connection() {
                    Ok(connection) => {
                        self.client = client;
                        self.connection = Some(connection);
                        self.connected = true;
                        Ok(true)
                    }
                    Err(e) => {
                        eprintln!("Warning: Could not connect to Redis: {}", e);
                        self.connected = false;
                        Ok(false)
                    }
                }
            }
            Err(e) => {
                eprintln!("Warning: Could not create Redis client: {}", e);
                self.connected = false;
                Ok(false)
            }
        }
    }

    pub fn is_connected(&self) -> bool {
        self.connected && self.connection.is_some()
    }

    /// Execute PING command and return result
    pub fn ping(&mut self) -> Result<String> {
        let conn = self.connection.as_mut().context("Not connected to Redis")?;
        let pong: String = redis::cmd("PING").query(conn)?;
        Ok(pong)
    }

    /// Try to reconnect - returns true if successful, false if fails
    pub fn reconnect(&mut self, url: &str) -> Result<bool> {
        let client = Client::open(url).context("Failed to create Redis client")?;
        match client.get_connection() {
            Ok(connection) => {
                self.client = client;
                self.connection = Some(connection);
                self.connected = true;
                self.current_url = url.to_string();
                Ok(true)
            }
            Err(e) => {
                // Keep the old connection if available, otherwise mark as disconnected
                self.connected = false;
                eprintln!("Warning: Could not reconnect to Redis: {}", e);
                Ok(false)
            }
        }
    }

    pub fn test_connection(url: &str) -> Result<()> {
        let client = Client::open(url).context("Failed to create Redis client")?;
        let mut conn = client
            .get_connection()
            .context("Failed to connect to Redis")?;

        // Quick PING to verify the connection works
        let pong: String = redis::cmd("PING").query(&mut conn)?;
        if pong != "PONG" {
            anyhow::bail!("Unexpected PING response: {}", pong);
        }

        Ok(())
    }

    pub fn fetch_keys(&mut self) -> Result<RedisData> {
        // Return empty data if not connected
        if !self.is_connected() {
            return Ok(RedisData {
                keys: Vec::new(),
                total_memory: 0,
                connected: false,
                error: Some("Not connected to Redis".to_string()),
            });
        }

        let conn = self.connection.as_mut().context("Not connected to Redis")?;

        let keys: Vec<String> = conn.keys("*")?;
        let mut key_infos = Vec::new();
        let mut total_memory = 0u64;

        for key_name in keys.iter().take(1000) {
            let key_type: String = redis::cmd("TYPE").arg(key_name).query(conn)?;

            let ttl: i64 = conn.ttl(key_name)?;

            let memory: u64 = redis::cmd("MEMORY")
                .arg("USAGE")
                .arg(key_name)
                .query(conn)
                .unwrap_or(0);

            let value_preview = match key_type.as_str() {
                "string" => {
                    let val: String = conn.get(key_name).unwrap_or_default();
                    if val.len() > 50 {
                        format!("{}...", &val[..47.min(val.len())])
                    } else {
                        val
                    }
                }
                "list" => {
                    let len: usize = conn.llen(key_name).unwrap_or(0);
                    format!("[{} items]", len)
                }
                "hash" => {
                    let len: usize = conn.hlen(key_name).unwrap_or(0);
                    format!("{{{} fields}}", len)
                }
                "set" => {
                    let len: usize = conn.scard(key_name).unwrap_or(0);
                    format!("[{} items]", len)
                }
                _ => "[binary data]".to_string(),
            };

            total_memory += memory;
            key_infos.push(KeyInfo {
                name: key_name.clone(),
                key_type,
                ttl,
                memory_bytes: memory,
                value_preview,
            });
        }

        self.total_memory = total_memory;
        self.connected = true;

        Ok(RedisData {
            keys: key_infos,
            total_memory,
            connected: true,
            error: None,
        })
    }

    /// Get the full value of a key (not just preview)
    pub fn get_full_value(&mut self, key_name: &str, key_type: &str) -> Result<String> {
        // Return error message if not connected
        if !self.is_connected() {
            return Ok(
                "Not connected to Redis. Use Tab to go to Settings and connect.".to_string(),
            );
        }

        let conn = self.connection.as_mut().context("Not connected to Redis")?;

        let value = match key_type {
            "string" => {
                let val: String = conn.get(key_name).unwrap_or_default();
                if val.is_empty() {
                    "(empty string)".to_string()
                } else {
                    // Try to format as JSON if it looks like JSON
                    try_format_as_json(&val)
                }
            }
            "list" => {
                let items: Vec<String> = conn.lrange(key_name, 0, -1).unwrap_or_default();
                if items.is_empty() {
                    "(empty list)".to_string()
                } else {
                    // Format each item, try to parse as JSON
                    items
                        .iter()
                        .enumerate()
                        .map(|(i, v)| {
                            let formatted = try_format_as_json(v);
                            if formatted.contains('\n') {
                                // If it has newlines after formatting, indent it
                                let indented: Vec<String> = formatted
                                    .lines()
                                    .map(|line| format!("    {}", line))
                                    .collect();
                                format!("[{}]\n{}", i, indented.join("\n"))
                            } else {
                                format!("[{}] {}", i, formatted)
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                }
            }
            "hash" => {
                let fields: Vec<(String, String)> = conn.hgetall(key_name).unwrap_or_default();
                if fields.is_empty() {
                    "(empty hash)".to_string()
                } else {
                    fields
                        .iter()
                        .map(|(k, v)| {
                            let formatted = try_format_as_json(v);
                            if formatted.contains('\n') {
                                // If value is JSON/object, pretty print it
                                format!("{}:\n{}", k, formatted)
                            } else {
                                format!("{}: {}", k, formatted)
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                }
            }
            "set" => {
                let members: Vec<String> = conn.smembers(key_name).unwrap_or_default();
                if members.is_empty() {
                    "(empty set)".to_string()
                } else {
                    members
                        .iter()
                        .enumerate()
                        .map(|(i, m)| {
                            let formatted = try_format_as_json(m);
                            if formatted.contains('\n') {
                                format!("[{}]\n{}", i, formatted)
                            } else {
                                format!("[{}] {}", i, formatted)
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                }
            }
            "zset" => {
                let members: Vec<(String, f64)> =
                    conn.zrange_withscores(key_name, 0, -1).unwrap_or_default();
                if members.is_empty() {
                    "(empty sorted set)".to_string()
                } else {
                    members
                        .iter()
                        .map(|(m, s)| {
                            let formatted = try_format_as_json(m);
                            if formatted.contains('\n') {
                                format!("{} (score: {})\n{}", formatted, s, formatted)
                            } else {
                                format!("{} (score: {})", formatted, s)
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                }
            }
            _ => "(unsupported type)".to_string(),
        };

        Ok(value)
    }
}
