use anyhow::Result;
use redis::{Client, aio::ConnectionManager};

pub struct Redis {
    pub manager: ConnectionManager,
}

impl Redis {
    pub async fn new(address: &str, password: Option<&str>) -> Result<Self> {
        let mut info = redis::IntoConnectionInfo::into_connection_info(address)?;

        if let Some(pass) = password {
            info.redis.password = Some(pass.to_string());
        }

        // Handle TLS and Insecure mode
        if address.starts_with("rediss://") {
            info.addr = match info.addr {
                redis::ConnectionAddr::Tcp(host, port) => redis::ConnectionAddr::TcpTls {
                    host,
                    port,
                    insecure: true,
                    tls_params: None,
                },
                redis::ConnectionAddr::TcpTls { host, port, .. } => redis::ConnectionAddr::TcpTls {
                    host,
                    port,
                    insecure: true,
                    tls_params: None,
                },
                addr => addr,
            };
        }

        let client = Client::open(info)?;
        let manager = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            client.get_connection_manager(),
        )
        .await
        .map_err(|_| anyhow::anyhow!("Redis connection timeout"))??;

        Ok(Self { manager })
    }

    /// Get a connection (cloned manager)
    pub fn get_connection(&self) -> ConnectionManager {
        self.manager.clone()
    }

    /// Execute a custom Redis command (useful for RedisJSON/Search)
    pub async fn cmd(&self, cmd_name: &str, args: Vec<&str>) -> Result<redis::Value> {
        let mut conn = self.get_connection();
        let mut cmd = redis::cmd(cmd_name);
        for arg in args {
            cmd.arg(arg);
        }

        let val: redis::Value = cmd.query_async(&mut conn).await?;
        Ok(val)
    }

    /// Setup RediSearch index for plugins (JSON based)
    #[allow(dead_code)]
    pub async fn initialize_indices(&self) -> Result<()> {
        let _ = self
            .cmd(
                "FT.CREATE",
                vec![
                    "idx:plugins",
                    "ON",
                    "JSON",
                    "PREFIX",
                    "1",
                    "plugin:",
                    "SCHEMA",
                    "$.id",
                    "AS",
                    "id",
                    "TEXT",
                    "$.name",
                    "AS",
                    "name",
                    "TEXT",
                    "$.capabilities[*]",
                    "AS",
                    "capabilities",
                    "TAG",
                ],
            )
            .await; // Ignore if exists

        Ok(())
    }
}
