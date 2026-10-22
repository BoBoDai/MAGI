//! External adapters for MAGI.

pub mod llm;

pub use llm::{
    ConnectionTestResult, EffectiveCapabilities, LlmCompat, LlmConfig, LlmEndpointError,
    LlmErrorKind, LlmModelList, MagiProvider, MaxTokensField, OpenAiCompatibleClient,
    ProviderProfileId, ProviderProfileInfo, ThinkingFormat, ThinkingMode, base_url_host,
    normalize_base_url, provider_catalog,
};

#[cfg(debug_assertions)]
pub use llm::DemoProvider;
