//! Runtime backend selection between the live client and the debug demo.

use std::sync::Arc;
use std::sync::RwLock;

#[cfg(debug_assertions)]
use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(debug_assertions)]
use magi_adapters::DemoProvider;
use magi_adapters::{
    ConnectionTestResult, LlmConfig, LlmEndpointError, LlmModelList, MagiProvider,
    OpenAiCompatibleClient,
};
use magi_core::{
    DebateCall, DebateReply, Opening, OpeningCall, Result, SessionSource, VoteCall, VoteCast,
};

pub struct LlmBackend {
    live: OpenAiCompatibleClient,
    #[cfg(debug_assertions)]
    demo: DemoProvider,
    #[cfg(debug_assertions)]
    demo_enabled: AtomicBool,
}

impl LlmBackend {
    pub fn new(config: Arc<RwLock<LlmConfig>>) -> Self {
        Self {
            live: OpenAiCompatibleClient::new(config),
            #[cfg(debug_assertions)]
            demo: DemoProvider::new(),
            #[cfg(debug_assertions)]
            demo_enabled: AtomicBool::new(false),
        }
    }

    pub fn demo_available(&self) -> bool {
        cfg!(debug_assertions)
    }

    #[cfg(debug_assertions)]
    pub fn demo_enabled(&self) -> bool {
        self.demo_enabled.load(Ordering::SeqCst)
    }

    #[cfg(not(debug_assertions))]
    pub fn demo_enabled(&self) -> bool {
        false
    }

    #[cfg(debug_assertions)]
    pub fn set_demo_mode(&self, enabled: bool) {
        self.demo_enabled.store(enabled, Ordering::SeqCst);
    }

    pub fn source(&self) -> SessionSource {
        if self.demo_enabled() {
            SessionSource::Mock
        } else {
            SessionSource::Live
        }
    }

    pub async fn fetch_models(
        &self,
        config: &LlmConfig,
    ) -> std::result::Result<LlmModelList, LlmEndpointError> {
        self.live.fetch_models(config).await
    }

    pub async fn test_connection(
        &self,
        config: &LlmConfig,
    ) -> std::result::Result<ConnectionTestResult, LlmEndpointError> {
        self.live.test_connection(config).await
    }
}

impl MagiProvider for LlmBackend {
    async fn opening_stream(
        &self,
        call: &OpeningCall,
        on_delta: &mut (dyn FnMut(String) + Send),
    ) -> Result<Opening> {
        #[cfg(debug_assertions)]
        if self.demo_enabled() {
            return self.demo.opening_stream(call, on_delta).await;
        }

        self.live.opening_stream(call, on_delta).await
    }

    async fn debate_stream(
        &self,
        call: &DebateCall,
        on_delta: &mut (dyn FnMut(String) + Send),
    ) -> Result<DebateReply> {
        #[cfg(debug_assertions)]
        if self.demo_enabled() {
            return self.demo.debate_stream(call, on_delta).await;
        }

        self.live.debate_stream(call, on_delta).await
    }

    async fn cast_vote(&self, call: &VoteCall) -> Result<VoteCast> {
        #[cfg(debug_assertions)]
        if self.demo_enabled() {
            return self.demo.cast_vote(call).await;
        }

        self.live.cast_vote(call).await
    }
}
