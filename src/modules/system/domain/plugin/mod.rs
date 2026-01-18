pub mod entity;
pub mod hooks;
pub mod registry;

#[cfg(test)]
mod tests;

pub use entity::{Capability, Manifest, MenuExtension, Plugin};
// Note: HookRegistry, PluginHook, HookContext are available via hooks module when needed
