//! Persistent LLM settings and write-only credentials.

mod manager;
mod normalize;
mod types;

#[cfg(test)]
mod tests;

pub use manager::SettingsManager;
pub use types::{LlmProbeInput, LlmSettingsInput, LlmSettingsView};

#[cfg(test)]
pub use types::{LlmProviderInput, LlmSettings};

pub const SETTINGS_SCHEMA_VERSION: u32 = 1;
