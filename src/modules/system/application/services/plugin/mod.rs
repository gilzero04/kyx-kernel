use anyhow::Result;
use crate::modules::system::domain::plugin::{Plugin, Manifest, PluginStatus};
use crate::modules::system::infrastructure::wasm_engine::WasmEngine;

use crate::core::infrastructure::redis::Redis;
use std::sync::Arc;
use serde_json::to_string;

pub struct PluginService {
    wasm_engine: WasmEngine,
    redis: Arc<Redis>,
}

impl PluginService {
    pub fn new(redis: Arc<Redis>) -> Self {
        Self {
            wasm_engine: WasmEngine::new(),
            redis,
        }
    }

    pub async fn install_plugin(&mut self, manifest: Manifest, wasm_path: &str) -> Result<Plugin> {
        // 1. Persistence via RedisJSON
        let key = format!("plugin:{}", manifest.id);
        let manifest_json = to_string(&manifest)?;
        
        // JSON.SET key $ json
        self.redis.cmd("JSON.SET", vec![&key, "$", &manifest_json]).await?;

        // 2. WASM Instantiation
        match self.wasm_engine.load(&manifest, wasm_path).await {
            Ok(_) => {
                println!("[System] Plugin {} initialized and persisted", manifest.name);
                Ok(Plugin {
                    manifest,
                    status: PluginStatus::Loaded,
                })
            }
            Err(e) => {
                println!("[System] Plugin {} persisted but failed to load WASM: {}", manifest.name, e);
                Ok(Plugin {
                    manifest,
                    status: PluginStatus::Error(e.to_string()),
                })
            }
        }
    }
}
