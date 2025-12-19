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

pub struct Plugin {
    pub manifest: Manifest,
    pub status: PluginStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PluginStatus {
    Installed,
    Loaded,
    Error(String),
}
