//! SSE framing and `<think>` filtering.
//!
//! Events are framed at the byte level and only decoded once complete, so a
//! multi-byte Chinese character split across network chunks is never mangled.

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(crate) struct StreamChunk {
    #[serde(default)]
    pub(crate) choices: Vec<StreamChoice>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct StreamChoice {
    #[serde(default)]
    pub(crate) delta: StreamDelta,
}

#[derive(Debug, Default, Deserialize)]
pub(crate) struct StreamDelta {
    #[serde(default)]
    pub(crate) content: Option<String>,
}

/// Buffers raw response bytes and yields complete SSE events.
#[derive(Default)]
pub(crate) struct SseBuffer {
    bytes: Vec<u8>,
}

impl SseBuffer {
    pub(crate) fn push(&mut self, chunk: &[u8]) -> Result<Vec<String>, String> {
        self.bytes.extend_from_slice(chunk);

        let mut events = Vec::new();
        while let Some((index, length)) = next_sse_event(&self.bytes) {
            let event: Vec<u8> = self.bytes.drain(..index + length).collect();
            let event = String::from_utf8(event)
                .map_err(|error| format!("SSE event is not valid UTF-8: {error}"))?;
            events.push(event);
        }

        Ok(events)
    }

    /// Decode an unterminated trailing event, if any.
    pub(crate) fn finish_remainder(&mut self) -> Result<Option<String>, String> {
        if self.bytes.is_empty() {
            return Ok(None);
        }

        let bytes = std::mem::take(&mut self.bytes);
        String::from_utf8(bytes)
            .map(Some)
            .map_err(|error| format!("SSE remainder is not valid UTF-8: {error}"))
    }
}

fn next_sse_event(buffer: &[u8]) -> Option<(usize, usize)> {
    let lf = find_subsequence(buffer, b"\n\n").map(|index| (index, 2));
    let crlf = find_subsequence(buffer, b"\r\n\r\n").map(|index| (index, 4));

    match (lf, crlf) {
        (Some(left), Some(right)) => Some(if left.0 <= right.0 { left } else { right }),
        (Some(left), None) => Some(left),
        (None, Some(right)) => Some(right),
        (None, None) => None,
    }
}

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }

    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

/// Holds back reasoning blocks and partial `<think` tags while streaming.
#[derive(Default)]
pub(crate) struct ThinkFilter {
    raw: String,
    emitted: usize,
    answer: String,
    saw_think: bool,
    thinking_done: bool,
}

impl ThinkFilter {
    pub(crate) fn push(&mut self, delta: &str) -> String {
        self.raw.push_str(delta);
        let mut out = String::new();

        loop {
            if self.thinking_done {
                if self.emitted < self.raw.len() {
                    out.push_str(&self.raw[self.emitted..]);
                    self.emitted = self.raw.len();
                }
                break;
            }

            if self.saw_think {
                if let Some(position) = self.raw[self.emitted..].find("</think") {
                    let position = self.emitted + position;
                    if let Some(close) = self.raw[position..].find('>') {
                        self.thinking_done = true;
                        self.emitted = position + close + 1;
                        continue;
                    }
                }
                break;
            }

            if let Some(position) = self.raw.find("<think") {
                if position > self.emitted {
                    out.push_str(&self.raw[self.emitted..position]);
                    self.emitted = position;
                }
                self.saw_think = true;
                continue;
            }

            let safe = safe_prefix_len(&self.raw);
            if safe > self.emitted {
                out.push_str(&self.raw[self.emitted..safe]);
                self.emitted = safe;
            }
            break;
        }

        self.answer.push_str(&out);
        out
    }

    pub(crate) fn finish(&self) -> String {
        if self.saw_think && !self.thinking_done {
            return String::new();
        }

        self.answer.trim().to_string()
    }
}

/// Largest prefix length that does not end with a partial `<think` tag.
fn safe_prefix_len(raw: &str) -> usize {
    const TAG: &str = "<think";
    let length = raw.len();

    for size in (1..TAG.len()).rev() {
        if raw.ends_with(&TAG[..size]) {
            return length - size;
        }
    }

    length
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frames_lf_and_crlf_events() {
        let mut buffer = SseBuffer::default();
        let events = buffer.push(b"data: a\r\n\r\ndata: b\n\npartial").unwrap();

        assert_eq!(events, vec!["data: a\r\n\r\n", "data: b\n\n"]);
        assert_eq!(
            buffer.finish_remainder().unwrap().as_deref(),
            Some("partial")
        );
    }

    #[test]
    fn decodes_utf8_split_across_chunks() {
        let mut buffer = SseBuffer::default();
        let text = "data: 你好\n\n";
        let bytes = text.as_bytes();

        assert!(buffer.push(&bytes[..8]).unwrap().is_empty());
        let events = buffer.push(&bytes[8..]).unwrap();
        assert_eq!(events, vec![text]);
    }

    #[test]
    fn filters_think_blocks() {
        let mut filter = ThinkFilter::default();
        assert_eq!(filter.push("hello "), "hello ");
        assert_eq!(filter.push("<thi"), "");
        assert_eq!(filter.push("nk>secret</think>world"), "world");
        assert_eq!(filter.finish(), "hello world");
    }

    #[test]
    fn holds_back_partial_think_tag() {
        let mut filter = ThinkFilter::default();
        assert_eq!(filter.push("ok<thi"), "ok");
        assert_eq!(filter.finish(), "ok");
    }
}
