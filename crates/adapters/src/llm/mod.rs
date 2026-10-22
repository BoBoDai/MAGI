//! LLM adapters: OpenAI-compatible transport, prompts, parsing, and a
//! debug-only demo provider.

mod client;
mod config;
#[cfg(debug_assertions)]
mod demo;
mod error;
mod parsing;
mod profiles;
mod prompts;
mod streaming;

#[cfg(test)]
mod tests;

use magi_core::{DebateCall, DebateReply, Opening, OpeningCall, Result, VoteCall, VoteCast};

pub use client::OpenAiCompatibleClient;
pub use config::{LlmConfig, base_url_host, normalize_base_url};
#[cfg(debug_assertions)]
pub use demo::DemoProvider;
pub use error::{ConnectionTestResult, LlmEndpointError, LlmErrorKind, LlmModelList};
pub use profiles::{
    EffectiveCapabilities, LlmCompat, MaxTokensField, ProviderProfileId, ProviderProfileInfo,
    ThinkingFormat, ThinkingMode, provider_catalog,
};

/// The three operations the MAGI protocol needs from a model backend.
#[allow(async_fn_in_trait)]
pub trait MagiProvider: Send + Sync {
    async fn opening_stream(
        &self,
        call: &OpeningCall,
        on_delta: &mut (dyn FnMut(String) + Send),
    ) -> Result<Opening>;

    async fn debate_stream(
        &self,
        call: &DebateCall,
        on_delta: &mut (dyn FnMut(String) + Send),
    ) -> Result<DebateReply>;

    async fn cast_vote(&self, call: &VoteCall) -> Result<VoteCast>;
}
