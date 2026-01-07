pub mod entity;
pub mod registry;
pub mod hooks;

#[cfg(test)]
mod tests;

pub use entity::{Manifest, Plugin, Capability};
pub use hooks::{HookRegistry, PluginHook, HookContext};
