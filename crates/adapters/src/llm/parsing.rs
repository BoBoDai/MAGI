//! Strict response parsing.
//!
//! Missing fields, wrong enum values, out-of-range confidence, empty required
//! text, and malformed JSON are all explicit errors. Nothing is silently
//! replaced with a default.

use magi_core::{DebateReply, Error, Opening, PersonaId, Result, Stance, VoteCast};
use serde::Deserialize;
use serde::de::DeserializeOwned;

#[derive(Debug, Deserialize)]
struct ChatResponse {
    #[serde(default)]
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    #[serde(default)]
    message: ChatMessage,
}

#[derive(Debug, Default, Deserialize)]
struct ChatMessage {
    #[serde(default)]
    content: Option<String>,
}

pub(crate) fn parse_chat_content(body: &str) -> Result<String> {
    let response: ChatResponse = serde_json::from_str(body)
        .map_err(|error| Error::InvalidLlmOutput(format!("invalid chat response: {error}")))?;

    let content = response
        .choices
        .into_iter()
        .next()
        .and_then(|choice| choice.message.content)
        .unwrap_or_default();
    let content = strip_think_block(&content).trim().to_string();

    if content.is_empty() {
        return Err(Error::InvalidLlmOutput(
            "LLM returned an empty response".into(),
        ));
    }

    Ok(content)
}

#[derive(Debug, Deserialize)]
struct ModelsResponse {
    #[serde(default)]
    data: Vec<ModelEntry>,
}

#[derive(Debug, Deserialize)]
struct ModelEntry {
    id: String,
}

pub(crate) fn parse_models_response(body: &str) -> Result<Vec<String>> {
    let response: ModelsResponse = serde_json::from_str(body)
        .map_err(|error| Error::InvalidLlmOutput(format!("invalid model list: {error}")))?;

    let mut models: Vec<String> = response
        .data
        .into_iter()
        .map(|entry| entry.id.trim().to_string())
        .filter(|id| !id.is_empty())
        .collect();
    models.sort();
    models.dedup();
    Ok(models)
}

/// Remove any `<think ...>...</think>` reasoning blocks.
///
/// An unterminated block is dropped from the tag onward: it is reasoning, not
/// answer content.
pub(crate) fn strip_think_block(content: &str) -> String {
    let mut text = content.trim().to_string();

    loop {
        let lower = text.to_ascii_lowercase();
        let Some(start) = lower.find("<think") else {
            break;
        };
        let Some(open_end) = text[start..].find('>').map(|index| start + index + 1) else {
            text.truncate(start);
            break;
        };
        let Some(close_start) = text[open_end..]
            .to_ascii_lowercase()
            .find("</think")
            .map(|index| open_end + index)
        else {
            text.truncate(start);
            break;
        };
        let Some(close_end) = text[close_start..]
            .find('>')
            .map(|index| close_start + index + 1)
        else {
            text.truncate(start);
            break;
        };
        text.replace_range(start..close_end, "");
    }

    text.trim().to_string()
}

/// Extract a JSON object from a raw or single-fenced response.
fn extract_json_payload(content: &str) -> Result<String> {
    let text = strip_think_block(content);
    let text = text.trim();

    if text.is_empty() {
        return Err(Error::InvalidLlmOutput("JSON payload is empty".into()));
    }

    if let Some(rest) = text.strip_prefix("```") {
        let rest = rest.strip_prefix("json").unwrap_or(rest);
        let rest = rest.trim_start();
        let end = rest.rfind("```").ok_or_else(|| {
            Error::InvalidLlmOutput("JSON payload has an unterminated code fence".into())
        })?;
        let inner = rest[..end].trim();
        if inner.is_empty() {
            return Err(Error::InvalidLlmOutput("JSON payload is empty".into()));
        }
        return Ok(inner.to_string());
    }

    Ok(text.to_string())
}

fn parse_payload<T: DeserializeOwned>(content: &str) -> Result<T> {
    let json = extract_json_payload(content)?;
    serde_json::from_str(&json).map_err(|error| {
        Error::InvalidLlmOutput(format!("JSON does not match the required schema: {error}"))
    })
}

fn required_field(name: &str, value: &str) -> Result<String> {
    let value = value.trim().to_string();
    if value.is_empty() {
        return Err(Error::InvalidLlmOutput(format!("{name} is empty")));
    }
    Ok(value)
}

fn bounded_text(name: &str, value: &str, max_chars: usize) -> Result<String> {
    let value = required_field(name, value)?;
    if value.chars().count() > max_chars {
        return Err(Error::InvalidLlmOutput(format!(
            "{name} exceeds {max_chars} characters"
        )));
    }
    Ok(value)
}

fn checked_confidence(confidence: u8) -> Result<u8> {
    if confidence > 100 {
        return Err(Error::InvalidLlmOutput(format!(
            "confidence {confidence} is out of range (0-100)"
        )));
    }
    Ok(confidence)
}

fn clean_bullets(name: &str, items: Vec<String>, min: usize, max: usize) -> Result<Vec<String>> {
    let items: Vec<String> = items
        .into_iter()
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
        .collect();

    if items.len() < min || items.len() > max {
        return Err(Error::InvalidLlmOutput(format!(
            "{name} must contain between {min} and {max} items"
        )));
    }

    for item in &items {
        if item.chars().count() > 300 {
            return Err(Error::InvalidLlmOutput(format!(
                "{name} item exceeds 300 characters"
            )));
        }
    }

    Ok(items)
}

#[derive(Debug, Deserialize)]
struct OpeningPayload {
    stance: Stance,
    confidence: u8,
    summary: String,
    #[serde(default)]
    key_points: Vec<String>,
    #[serde(default)]
    risks: Vec<String>,
}

pub(crate) fn parse_opening(content: &str, persona: PersonaId, model: &str) -> Result<Opening> {
    let payload: OpeningPayload = parse_payload(content)?;

    Ok(Opening {
        persona,
        stance: payload.stance,
        confidence: checked_confidence(payload.confidence)?,
        summary: bounded_text("summary", &payload.summary, 1_200)?,
        key_points: clean_bullets("key_points", payload.key_points, 1, 8)?,
        risks: clean_bullets("risks", payload.risks, 1, 8)?,
        model: bounded_text("model", model, 200)?,
    })
}

pub(crate) fn parse_debate_reply(
    content: &str,
    persona: PersonaId,
    model: &str,
) -> Result<DebateReply> {
    let text = strip_think_block(content);

    Ok(DebateReply {
        persona,
        text: bounded_text("debate reply", &text, 6_000)?,
        model: bounded_text("model", model, 200)?,
    })
}

#[derive(Debug, Deserialize)]
struct VotePayload {
    stance: Stance,
    confidence: u8,
    reason: String,
}

pub(crate) fn parse_vote(content: &str, persona: PersonaId, model: &str) -> Result<VoteCast> {
    let payload: VotePayload = parse_payload(content)?;

    Ok(VoteCast {
        persona,
        stance: payload.stance,
        confidence: checked_confidence(payload.confidence)?,
        reason: bounded_text("reason", &payload.reason, 800)?,
        model: bounded_text("model", model, 200)?,
    })
}
