pub mod entity;
pub mod registry;
pub mod hooks;

#[cfg(test)]
mod tests;

pub use entity::{Manifest, Plugin, Capability};
// Note: HookRegistry, PluginHook, HookContext are available via hooks module when needed
