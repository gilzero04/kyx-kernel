use anyhow::{Result, anyhow};
use wasmer::{Instance, Module, Store, Cranelift};
use crate::modules::system::domain::plugin::Manifest;
use std::fs;

pub struct WasmEngine;

impl WasmEngine {
    pub fn new() -> Self {
        Self
    }

    pub async fn load(&mut self, manifest: &Manifest, path: &str) -> Result<Instance> {
        println!("[System] Loading WASM Plugin: {} (v{})", manifest.name, manifest.version);
        
        let engine = wasmer::sys::EngineBuilder::new(Cranelift::default()).engine();
        let mut store = Store::new(engine);
        
        // 1. Read WASM bytes
        let wasm_bytes = fs::read(path)
            .map_err(|e| anyhow!("Failed to read WASM file at {}: {}", path, e))?;

        // 2. Compile Module
        let module = Module::new(&store, wasm_bytes)
            .map_err(|e| anyhow!("Failed to compile WASM module: {}", e))?;

        // 3. Create Import Object
        let import_object = wasmer::imports! {};

        // 4. Instantiate
        let instance = Instance::new(&mut store, &module, &import_object)
            .map_err(|e| anyhow!("Failed to instantiate WASM module: {}", e))?;

        Ok(instance)
    }
}
