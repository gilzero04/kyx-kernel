use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub capabilities: HashSet<Capability>,
    pub runtime: RuntimeType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Capability {
    Api,
    Ui,
    Storage,
    Event,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RuntimeType {
    Wasm,
    Service,
}

#[allow(dead_code)]
pub struct Plugin {
    #[allow(dead_code)]
    pub manifest: Manifest,
    #[allow(dead_code)]
    pub status: PluginStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PluginStatus {
    #[allow(dead_code)]
    Installed,
    #[allow(dead_code)]
    Loaded,
    #[allow(dead_code)]
    Error(String),
}
