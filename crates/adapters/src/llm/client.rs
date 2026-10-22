//! OpenAI-compatible chat client with SSE streaming.

use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use futures_util::StreamExt;
use magi_core::{DebateCall, DebateReply, Error, Opening, OpeningCall, Result, VoteCall, VoteCast};
use serde_json::Value;

use super::MagiProvider;
use super::config::{LlmConfig, normalize_base_url};
use super::error::{
    ConnectionTestResult, LlmEndpointError, LlmModelList, map_http_error, request_id_from_headers,
    transport_error,
};
use super::parsing::{
    parse_chat_content, parse_debate_reply, parse_models_response, parse_opening, parse_vote,
};
use super::profiles::{ThinkingFormat, ThinkingMode};
use super::prompts::{debate_messages, opening_messages, vote_messages};
use super::streaming::{SseBuffer, StreamChunk, ThinkFilter};

pub struct OpenAiCompatibleClient {
    http: reqwest::Client,
    config: Arc<RwLock<LlmConfig>>,
}

impl OpenAiCompatibleClient {
    pub fn new(config: Arc<RwLock<LlmConfig>>) -> Self {
        let http = reqwest::Client::builder()
            .user_agent(concat!("MAGI/", env!("CARGO_PKG_VERSION")))
            .build()
            .expect("failed to build the HTTP client");

        Self { http, config }
    }

    fn current_config(&self) -> Result<LlmConfig> {
        self.config
            .read()
            .map(|config| config.clone())
            .map_err(|_| Error::Config("LLM 配置锁已损坏".into()))
    }

    async fn chat(
        &self,
        config: &LlmConfig,
        messages: Vec<Value>,
        max_tokens: u32,
        timeout: Duration,
        json_mode: bool,
        temperature: f64,
    ) -> Result<String> {
        validate(config)?;
        let base_url = normalize_base_url(&config.base_url).map_err(Error::Config)?;
        let body = build_chat_body(config, messages, max_tokens, json_mode, false, temperature);

        let response = self
            .http
            .post(format!("{base_url}/chat/completions"))
            .bearer_auth(config.api_key.trim())
            .timeout(timeout)
            .json(&body)
            .send()
            .await
            .map_err(|error| {
                Error::Llm(transport_error("LLM request failed", error).to_string())
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let headers = response.headers().clone();
            let body = response.text().await.unwrap_or_default();
            return Err(Error::Llm(
                map_http_error(status, &headers, &body).to_string(),
            ));
        }

        let body = response.text().await.map_err(|error| {
            Error::Llm(transport_error("failed to read LLM response", error).to_string())
        })?;

        parse_chat_content(&body)
    }

    async fn stream_chat(
        &self,
        config: &LlmConfig,
        messages: Vec<Value>,
        max_tokens: u32,
        timeout: Duration,
        on_delta: &mut (dyn FnMut(String) + Send),
        temperature: f64,
    ) -> Result<String> {
        validate(config)?;
        let base_url = normalize_base_url(&config.base_url).map_err(Error::Config)?;
        let body = build_chat_body(config, messages, max_tokens, false, true, temperature);

        let response = self
            .http
            .post(format!("{base_url}/chat/completions"))
            .bearer_auth(config.api_key.trim())
            .timeout(timeout)
            .json(&body)
            .send()
            .await
            .map_err(|error| Error::Llm(transport_error("LLM stream failed", error).to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let headers = response.headers().clone();
            let body = response.text().await.unwrap_or_default();
            return Err(Error::Llm(
                map_http_error(status, &headers, &body).to_string(),
            ));
        }

        let mut stream = response.bytes_stream();
        let mut buffer = SseBuffer::default();
        let mut full_body: Vec<u8> = Vec::new();
        let mut filter = ThinkFilter::default();
        let mut saw_event = false;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|error| {
                Error::Llm(transport_error("failed to read LLM stream", error).to_string())
            })?;
            full_body.extend_from_slice(&chunk);

            let events = buffer.push(&chunk).map_err(Error::Llm)?;
            for event in &events {
                apply_sse_event(event, &mut filter, &mut saw_event, on_delta);
            }
        }

        if let Some(remainder) = buffer.finish_remainder().map_err(Error::Llm)? {
            apply_sse_event(&remainder, &mut filter, &mut saw_event, on_delta);
        }

        if !saw_event {
            // Some gateways ignore stream=true and return a normal completion.
            let body = String::from_utf8_lossy(&full_body);
            let content = parse_chat_content(&body)?;
            on_delta(content.clone());
            return Ok(content);
        }

        let answer = filter.finish();
        if answer.is_empty() {
            return Err(Error::InvalidLlmOutput(
                "LLM returned an empty response".into(),
            ));
        }

        Ok(answer)
    }

    pub async fn fetch_models(
        &self,
        config: &LlmConfig,
    ) -> std::result::Result<LlmModelList, LlmEndpointError> {
        if !config.is_configured() {
            return Err(LlmEndpointError::missing("请先在设置中填写 API key"));
        }
        let base_url = normalize_base_url(&config.base_url).map_err(LlmEndpointError::missing)?;

        let response = self
            .http
            .get(format!("{base_url}/models"))
            .bearer_auth(config.api_key.trim())
            .timeout(Duration::from_secs(30))
            .send()
            .await
            .map_err(|error| transport_error("failed to fetch models", error))?;

        let status = response.status();
        let headers = response.headers().clone();
        let body = response.text().await.unwrap_or_default();

        if !status.is_success() {
            return Err(map_http_error(status, &headers, &body));
        }

        let request_id = request_id_from_headers(&headers);
        let models = parse_models_response(&body)
            .map_err(|error| LlmEndpointError::invalid_response(error.to_string()))?;

        Ok(LlmModelList { models, request_id })
    }

    pub async fn test_connection(
        &self,
        config: &LlmConfig,
    ) -> std::result::Result<ConnectionTestResult, LlmEndpointError> {
        if !config.is_configured() {
            return Err(LlmEndpointError::missing("请先在设置中填写 API key"));
        }
        let base_url = normalize_base_url(&config.base_url).map_err(LlmEndpointError::missing)?;
        let messages = vec![serde_json::json!({
            "role": "user",
            "content": "ping",
        })];
        let body = build_chat_body(config, messages, 16, false, false, 0.2);

        let started = Instant::now();
        let response = self
            .http
            .post(format!("{base_url}/chat/completions"))
            .bearer_auth(config.api_key.trim())
            .timeout(Duration::from_secs(45))
            .json(&body)
            .send()
            .await
            .map_err(|error| transport_error("connection test failed", error))?;

        let status = response.status();
        let headers = response.headers().clone();
        let body = response.text().await.unwrap_or_default();

        if !status.is_success() {
            return Err(map_http_error(status, &headers, &body));
        }

        let _ = body;
        Ok(ConnectionTestResult {
            ok: true,
            latency_ms: started.elapsed().as_millis() as u64,
            model: config.model.trim().to_string(),
            request_id: request_id_from_headers(&headers),
        })
    }
}

impl MagiProvider for OpenAiCompatibleClient {
    async fn opening_stream(
        &self,
        call: &OpeningCall,
        on_delta: &mut (dyn FnMut(String) + Send),
    ) -> Result<Opening> {
        let config = self.current_config()?;
        tracing::info!(
            persona = call.persona.as_str(),
            model = %config.model,
            "collecting opening statement"
        );

        let content = self
            .stream_chat(
                &config,
                opening_messages(call),
                2048,
                Duration::from_secs(120),
                on_delta,
                call.persona.temperature(),
            )
            .await?;

        parse_opening(&content, call.persona, config.model.trim())
    }

    async fn debate_stream(
        &self,
        call: &DebateCall,
        on_delta: &mut (dyn FnMut(String) + Send),
    ) -> Result<DebateReply> {
        let config = self.current_config()?;
        tracing::info!(
            persona = call.persona.as_str(),
            model = %config.model,
            "collecting debate reply"
        );

        let content = self
            .stream_chat(
                &config,
                debate_messages(call),
                1200,
                Duration::from_secs(120),
                on_delta,
                call.persona.temperature(),
            )
            .await?;

        parse_debate_reply(&content, call.persona, config.model.trim())
    }

    async fn cast_vote(&self, call: &VoteCall) -> Result<VoteCast> {
        let config = self.current_config()?;
        tracing::info!(
            persona = call.persona.as_str(),
            model = %config.model,
            "collecting final ballot"
        );

        let content = self
            .chat(
                &config,
                vote_messages(call),
                512,
                Duration::from_secs(90),
                true,
                call.persona.temperature(),
            )
            .await?;

        parse_vote(&content, call.persona, config.model.trim())
    }
}

fn validate(config: &LlmConfig) -> Result<()> {
    if !config.is_configured() {
        return Err(Error::Config("请先在设置中填写 API key".into()));
    }
    if config.model.trim().is_empty() {
        return Err(Error::Config("LLM 模型名不能为空".into()));
    }
    Ok(())
}

fn apply_sse_event(
    event: &str,
    filter: &mut ThinkFilter,
    saw_event: &mut bool,
    on_delta: &mut (dyn FnMut(String) + Send),
) {
    for line in event.lines() {
        let Some(data) = line.strip_prefix("data:") else {
            continue;
        };
        let data = data.trim();
        if data.is_empty() {
            continue;
        }

        *saw_event = true;
        if data == "[DONE]" {
            continue;
        }

        match serde_json::from_str::<StreamChunk>(data) {
            Ok(parsed) => {
                if let Some(content) = parsed
                    .choices
                    .into_iter()
                    .next()
                    .and_then(|choice| choice.delta.content)
                {
                    let out = filter.push(&content);
                    if !out.is_empty() {
                        on_delta(out);
                    }
                }
            }
            Err(error) => {
                tracing::warn!(%error, data, "ignoring malformed SSE chunk");
            }
        }
    }
}

pub(crate) fn build_chat_body(
    config: &LlmConfig,
    messages: Vec<Value>,
    max_tokens: u32,
    json_mode: bool,
    stream: bool,
    temperature: f64,
) -> Value {
    let capabilities = config.capabilities();

    let mut body = serde_json::json!({
        "model": config.model.trim(),
        "messages": messages,
        "temperature": temperature,
        "stream": stream,
    });

    body[capabilities.max_tokens_field.as_str()] = serde_json::json!(max_tokens);

    if json_mode && capabilities.json_mode {
        body["response_format"] = serde_json::json!({ "type": "json_object" });
    }

    match (capabilities.thinking_mode, capabilities.thinking_format) {
        (ThinkingMode::Default, _) => {}
        (ThinkingMode::Disabled, ThinkingFormat::Thinking) => {
            body["thinking"] = serde_json::json!({ "type": "disabled" });
        }
        (ThinkingMode::Disabled, ThinkingFormat::EnableThinking) => {
            body["enable_thinking"] = serde_json::json!(false);
        }
        (ThinkingMode::Adaptive, ThinkingFormat::Thinking) => {
            body["thinking"] = serde_json::json!({ "type": "adaptive" });
        }
        (ThinkingMode::Adaptive, ThinkingFormat::EnableThinking) => {
            body["enable_thinking"] = serde_json::json!(true);
        }
    }

    body
}
