// ════════════════════════════════════════════════════════════════════════════
// Plugin Service - Legacy Wrapper (Deprecated)
// Use PluginRegistry instead for new implementations
// ════════════════════════════════════════════════════════════════════════════

use anyhow::Result;
use crate::modules::system::domain::plugin::{Plugin, Manifest};
use crate::modules::system::infrastructure::wasm_engine::WasmEngine;
use crate::core::infrastructure::redis::Redis;
use std::sync::Arc;
use serde_json::to_string;

/// Legacy Plugin Service
/// @deprecated Use PluginRegistry for new implementations
#[allow(dead_code)]
pub struct PluginService {
    wasm_engine: WasmEngine,
    redis: Arc<Redis>,
}

impl PluginService {
    #[allow(dead_code)]
    pub fn new(redis: Arc<Redis>) -> Self {
        Self {
            wasm_engine: WasmEngine::new(),
            redis,
        }
    }

    /// Legacy install method - creates minimal plugin record
    #[allow(dead_code)]
    #[deprecated(note = "Use PluginRegistry::install instead")]
    pub async fn install_plugin(&self, manifest: Manifest, wasm_path: &str) -> Result<Plugin> {
        // 1. Persistence via RedisJSON
        let key = format!("plugin:{}", manifest.id);
        let manifest_json = to_string(&manifest)?;
        
        // JSON.SET key $ json
        self.redis.cmd("JSON.SET", vec![&key, "$", &manifest_json]).await?;

        // 2. WASM Instantiation
        let _status = match self.wasm_engine.load(&manifest, wasm_path).await {
            Ok(_) => {
                log::info!("[System] Plugin {} initialized and persisted", manifest.name);
                "enabled".to_string()
            }
            Err(e) => {
                log::warn!("[System] Plugin {} persisted but failed to load WASM: {}", manifest.name, e);
                "error".to_string()
            }
        };
        
        // Create minimal plugin record
        let plugin = Plugin::from_manifest(&manifest, None);
        Ok(plugin)
    }
}
