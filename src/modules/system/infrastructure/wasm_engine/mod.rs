// ════════════════════════════════════════════════════════════════════════════
// WASM Engine - Plugin Runtime Environment
// ════════════════════════════════════════════════════════════════════════════

pub mod host_functions;

use anyhow::{Result, anyhow};
use wasmer::{Instance, Module, Store, Cranelift, imports};
use log::info;
use std::sync::Arc;
use std::fs;

pub use host_functions::HostFunctions;

/// WASM Engine for loading and running plugin modules
pub struct WasmEngine {
    host_functions: Arc<HostFunctions>,
}

impl WasmEngine {
    pub fn new() -> Self {
        Self {
            host_functions: Arc::new(HostFunctions::new()),
        }
    }
    
    /// Create with shared host functions
    pub fn with_host_functions(host_functions: Arc<HostFunctions>) -> Self {
        Self { host_functions }
    }
    
    /// Get reference to host functions
    pub fn host_functions(&self) -> &Arc<HostFunctions> {
        &self.host_functions
    }
    
    /// Load a WASM module from file path
    pub async fn load_from_path(&self, path: &str) -> Result<Instance> {
        info!("Loading WASM from path: {}", path);
        
        let wasm_bytes = fs::read(path)
            .map_err(|e| anyhow!("Failed to read WASM file at {}: {}", path, e))?;
        
        self.load_from_bytes(&wasm_bytes).await
    }
    
    /// Load a WASM module from bytes
    pub async fn load_from_bytes(&self, wasm_bytes: &[u8]) -> Result<Instance> {
        let engine = wasmer::sys::EngineBuilder::new(Cranelift::default()).engine();
        let mut store = Store::new(engine);
        
        // Compile Module
        let module = Module::new(&store, wasm_bytes)
            .map_err(|e| anyhow!("Failed to compile WASM module: {}", e))?;
        
        // Create Import Object with host functions
        // Note: Full WASM↔Host binding requires wasmer_wasi or custom linking
        // This is a simplified version - production would use wasmer-wasi
        let import_object = imports! {
            // Host functions are accessed through HostFunctions struct
            // rather than direct WASM imports for now
            // 
            // In production with wasmer-wasi:
            // "env" => {
            //     "kv_get" => Function::new_typed(&mut store, |ptr: i32, len: i32| -> i64 { ... }),
            //     "kv_set" => Function::new_typed(&mut store, |k_ptr: i32, k_len: i32, v_ptr: i32, v_len: i32| -> i32 { ... }),
            //     "log_info" => Function::new_typed(&mut store, |ptr: i32, len: i32| { ... }),
            //     "emit_event" => Function::new_typed(&mut store, |type_ptr: i32, type_len: i32, payload_ptr: i32, payload_len: i32| -> i32 { ... }),
            // }
        };
        
        // Instantiate
        let instance = Instance::new(&mut store, &module, &import_object)
            .map_err(|e| anyhow!("Failed to instantiate WASM module: {}", e))?;
        
        info!("WASM module loaded successfully");
        Ok(instance)
    }
    
    /// Load a WASM module with manifest (legacy method)
    #[allow(dead_code)]
    pub async fn load(&self, manifest: &crate::modules::system::domain::plugin::Manifest, path: &str) -> Result<Instance> {
        info!("Loading WASM Plugin: {} (v{})", manifest.name, manifest.version);
        self.load_from_path(path).await
    }
    
    /// Execute a function from a loaded WASM instance
    #[allow(dead_code)]
    pub fn call_function(&self, instance: &Instance, store: &mut Store, func_name: &str) -> Result<()> {
        let func = instance.exports.get_function(func_name)
            .map_err(|e| anyhow!("Function '{}' not found: {}", func_name, e))?;
        
        func.call(store, &[])
            .map_err(|e| anyhow!("Failed to call '{}': {}", func_name, e))?;
        
        Ok(())
    }
    
    /// Cleanup plugin data from host functions
    pub fn cleanup_plugin(&self, plugin_id: &str) {
        self.host_functions.cleanup_plugin(plugin_id);
    }
}

impl Default for WasmEngine {
    fn default() -> Self {
        Self::new()
    }
}
