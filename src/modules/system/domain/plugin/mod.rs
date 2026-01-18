pub mod entity;
pub mod hooks;
pub mod registry;

#[cfg(test)]
mod tests;

pub use entity::{Capability, Manifest, Plugin};
// Note: HookRegistry, PluginHook, HookContext are available via hooks module when needed
// Note: MenuExtension is available via entity module for tests
