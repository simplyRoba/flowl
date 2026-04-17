use std::time::Duration;

use async_trait::async_trait;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use reqwest::Client;
use serde_json::{Value, json};
use tokio_stream::StreamExt as _;
use tracing::{debug, warn};

const CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const REQUEST_TIMEOUT: Duration = Duration::from_mins(2);
const MAX_SSE_LINE_LEN: usize = 64 * 1024;

use super::prompts;
use super::provider::AiProvider;
use super::types::{ChatMessage, ChatResponseStream, IdentifyResponse};

pub struct OpenAiProvider {
    client: Client,
    api_key: String,
    base_url: String,
    model: String,
}

impl OpenAiProvider {
    /// # Panics
    /// Panics if the HTTP client builder fails (should never happen with default TLS).
    pub fn new(api_key: String, base_url: String, model: String) -> Self {
        Self {
            client: Client::builder()
                .connect_timeout(CONNECT_TIMEOUT)
                .build()
                .expect("failed to build HTTP client"),
            api_key,
            base_url,
            model,
        }
    }

    fn completions_url(&self) -> String {
        format!("{}/chat/completions", self.base_url)
    }
}

#[async_trait]
impl AiProvider for OpenAiProvider {
    async fn identify(
        &self,
        images: &[&[u8]],
        locale: &str,
    ) -> Result<IdentifyResponse, Box<dyn std::error::Error + Send + Sync>> {
        let prompt_text = prompts::build_identify_prompt(locale);

        let mut content: Vec<Value> = vec![json!({
            "type": "text",
            "text": prompt_text
        })];

        for image_data in images {
            let b64 = STANDARD.encode(image_data);
            content.push(json!({
                "type": "image_url",
                "image_url": {
                    "url": format!("data:image/jpeg;base64,{b64}")
                }
            }));
        }

        let identify_item_schema = json!({
            "type": "object",
            "properties": {
                "common_name": { "type": "string" },
                "scientific_name": { "type": "string" },
                "confidence": { "type": ["number", "null"] },
                "summary": { "type": ["string", "null"] },
                "care_profile": {
                    "anyOf": [
                        {
                            "type": "object",
                            "properties": {
                                "watering_interval_days": { "type": ["integer", "null"] },
                                "light_needs": { "anyOf": [{ "type": "string", "enum": ["direct", "indirect", "low"] }, { "type": "null" }] },
                                "difficulty": { "anyOf": [{ "type": "string", "enum": ["easy", "moderate", "demanding"] }, { "type": "null" }] },
                                "pet_safety": { "anyOf": [{ "type": "string", "enum": ["safe", "caution", "toxic"] }, { "type": "null" }] },
                                "growth_speed": { "anyOf": [{ "type": "string", "enum": ["slow", "moderate", "fast"] }, { "type": "null" }] },
                                "soil_type": { "anyOf": [{ "type": "string", "enum": ["standard", "cactus-mix", "orchid-bark", "peat-moss"] }, { "type": "null" }] },
                                "soil_moisture": { "anyOf": [{ "type": "string", "enum": ["dry", "moderate", "moist"] }, { "type": "null" }] }
                            },
                            "required": ["watering_interval_days", "light_needs", "difficulty", "pet_safety", "growth_speed", "soil_type", "soil_moisture"],
                            "additionalProperties": false
                        },
                        { "type": "null" }
                    ]
                }
            },
            "required": ["common_name", "scientific_name", "confidence", "summary", "care_profile"],
            "additionalProperties": false
        });

        let body = json!({
            "model": self.model,
            "messages": [{
                "role": "user",
                "content": content
            }],
            "response_format": {
                "type": "json_schema",
                "json_schema": {
                    "name": "identify_response",
                    "strict": true,
                    "schema": {
                        "type": "object",
                        "properties": {
                            "suggestions": {
                                "type": "array",
                                "items": identify_item_schema
                            },
                            "rejected": { "type": "boolean" },
                            "rejected_reason": { "type": ["string", "null"] }
                        },
                        "required": ["suggestions", "rejected", "rejected_reason"],
                        "additionalProperties": false
                    }
                }
            }
        });

        let response = self
            .client
            .post(self.completions_url())
            .bearer_auth(&self.api_key)
            .timeout(REQUEST_TIMEOUT)
            .json(&body)
            .send()
            .await?
            .error_for_status()?;

        let response_body: Value = response.json().await?;
        let content_str = response_body["choices"][0]["message"]["content"]
            .as_str()
            .ok_or("Missing content in API response")?;

        debug!(raw_content = %content_str, "AI raw response content");

        let result: IdentifyResponse = serde_json::from_str(content_str)?;
        Ok(result)
    }

    async fn chat(
        &self,
        system_prompt: &str,
        messages: &[ChatMessage],
        image: Option<&[u8]>,
        _locale: &str,
    ) -> Result<ChatResponseStream, Box<dyn std::error::Error + Send + Sync>> {
        let api_messages = build_chat_messages(system_prompt, messages, image);

        let body = json!({
            "model": self.model,
            "messages": api_messages,
            "stream": true
        });

        let response = self
            .client
            .post(self.completions_url())
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await?
            .error_for_status()?;

        let (tx, rx) = tokio::sync::mpsc::channel::<Result<String, String>>(32);

        tokio::spawn(stream_sse_deltas(response, tx));

        Ok(tokio_stream::wrappers::ReceiverStream::new(rx))
    }

    async fn summarize(
        &self,
        system_prompt: &str,
        messages: &[ChatMessage],
        _locale: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut api_messages: Vec<Value> = vec![json!({
            "role": "system",
            "content": system_prompt
        })];

        for msg in messages {
            api_messages.push(json!({
                "role": msg.role,
                "content": msg.content
            }));
        }

        let body = json!({
            "model": self.model,
            "messages": api_messages,
            "response_format": {
                "type": "json_schema",
                "json_schema": {
                    "name": "summarize_response",
                    "strict": true,
                    "schema": {
                        "type": "object",
                        "properties": {
                            "summary": { "type": "string" }
                        },
                        "required": ["summary"],
                        "additionalProperties": false
                    }
                }
            }
        });

        let response = self
            .client
            .post(self.completions_url())
            .bearer_auth(&self.api_key)
            .timeout(REQUEST_TIMEOUT)
            .json(&body)
            .send()
            .await?
            .error_for_status()?;

        let response_body: Value = response.json().await?;
        let content_str = response_body["choices"][0]["message"]["content"]
            .as_str()
            .ok_or("Missing content in API response")?;

        debug!(raw_content = %content_str, "AI summarize raw response");

        let parsed: Value = serde_json::from_str(content_str)?;
        let summary = parsed["summary"]
            .as_str()
            .ok_or("Missing 'summary' field in AI response")?;

        Ok(summary.to_string())
    }
}

/// Build the message array for a chat completion request.
///
/// The resulting array is structured as: system prompt, then history/user messages in order.
/// This layout is important for `OpenAI` automatic prompt caching: on follow-up turns the
/// prefix (system prompt + earlier messages) is byte-identical to the previous request, so
/// the cached prefix can be reused.
fn build_chat_messages(
    system_prompt: &str,
    messages: &[ChatMessage],
    image: Option<&[u8]>,
) -> Vec<Value> {
    let mut api_messages: Vec<Value> = vec![json!({
        "role": "system",
        "content": system_prompt
    })];

    // Add history messages
    for msg in messages {
        if let Some(ref img_b64) = msg.image {
            // Message with image: use content array
            api_messages.push(json!({
                "role": msg.role,
                "content": [
                    { "type": "text", "text": msg.content },
                    { "type": "image_url", "image_url": { "url": format!("data:image/jpeg;base64,{img_b64}") } }
                ]
            }));
        } else {
            api_messages.push(json!({
                "role": msg.role,
                "content": msg.content
            }));
        }
    }

    // Add current image to the last user message if provided
    if let Some(img_bytes) = image {
        let b64 = STANDARD.encode(img_bytes);
        if let Some(last) = api_messages.last_mut()
            && last["role"] == "user"
        {
            let text = last["content"].as_str().unwrap_or("").to_string();
            last["content"] = json!([
                { "type": "text", "text": text },
                { "type": "image_url", "image_url": { "url": format!("data:image/jpeg;base64,{b64}") } }
            ]);
        }
    }

    api_messages
}

/// Read an SSE byte stream from the `OpenAI` API, extract content deltas, and forward them
/// through the channel.
async fn stream_sse_deltas(
    response: reqwest::Response,
    tx: tokio::sync::mpsc::Sender<Result<String, String>>,
) {
    let mut stream = response.bytes_stream();
    let mut buf = String::new();
    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(bytes) => {
                buf.push_str(&String::from_utf8_lossy(&bytes));
                if buf.len() > MAX_SSE_LINE_LEN {
                    let _ = tx.send(Err("SSE line too long".to_string())).await;
                    return;
                }
                while let Some(pos) = buf.find('\n') {
                    if let Some(delta) = parse_sse_line(&buf[..pos])
                        && tx.send(Ok(delta)).await.is_err()
                    {
                        return;
                    }
                    buf = buf[pos + 1..].to_string();
                }
            }
            Err(e) => {
                let _ = tx.send(Err(e.to_string())).await;
                return;
            }
        }
    }
}

/// Parse a single SSE line from the `OpenAI` streaming response.
/// Returns `Some(delta_text)` if the line contains a content delta, `None` otherwise.
fn parse_sse_line(line: &str) -> Option<String> {
    let data = line.strip_prefix("data: ")?;

    // Skip the [DONE] marker and empty data
    if data == "[DONE]" || data.is_empty() {
        return None;
    }

    match serde_json::from_str::<Value>(data) {
        Ok(parsed) => parsed["choices"][0]["delta"]["content"]
            .as_str()
            .filter(|s| !s.is_empty())
            .map(String::from),
        Err(e) => {
            warn!(line = %line, error = %e, "Failed to parse SSE chunk");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn completions_url_default_base() {
        let provider = OpenAiProvider::new(
            "key".into(),
            "https://api.openai.com/v1".into(),
            "gpt-4.1-mini".into(),
        );
        assert_eq!(
            provider.completions_url(),
            "https://api.openai.com/v1/chat/completions"
        );
    }

    #[test]
    fn completions_url_custom_base() {
        let provider = OpenAiProvider::new(
            "key".into(),
            "http://localhost:11434/v1".into(),
            "llama3".into(),
        );
        assert_eq!(
            provider.completions_url(),
            "http://localhost:11434/v1/chat/completions"
        );
    }

    #[test]
    fn identify_request_payload_structure() {
        let image_data: &[u8] = b"fake-image-bytes";
        let b64 = STANDARD.encode(image_data);

        let mut content: Vec<Value> = vec![json!({
            "type": "text",
            "text": "Identify this plant from the photo(s). Provide your top 3 most likely identifications, ranked by confidence (highest first). For each, include the common name, scientific name, your confidence level, a short summary of the species, and a care profile with typical care requirements. Respond in English."
        })];
        content.push(json!({
            "type": "image_url",
            "image_url": {
                "url": format!("data:image/jpeg;base64,{b64}")
            }
        }));

        let identify_item_schema = json!({
            "type": "object",
            "properties": {
                "common_name": { "type": "string" },
                "scientific_name": { "type": "string" }
            },
            "required": ["common_name", "scientific_name"],
            "additionalProperties": false
        });

        let body = json!({
            "model": "gpt-4.1-mini",
            "messages": [{
                "role": "user",
                "content": content
            }],
            "response_format": {
                "type": "json_schema",
                "json_schema": {
                    "name": "identify_response",
                    "strict": true,
                    "schema": {
                        "type": "object",
                        "properties": {
                            "suggestions": {
                                "type": "array",
                                "items": identify_item_schema
                            },
                            "rejected": { "type": "boolean" },
                            "rejected_reason": { "type": ["string", "null"] }
                        },
                        "required": ["suggestions", "rejected", "rejected_reason"],
                        "additionalProperties": false
                    }
                }
            }
        });

        assert_eq!(body["model"], "gpt-4.1-mini");
        assert_eq!(body["response_format"]["type"], "json_schema");
        assert_eq!(
            body["response_format"]["json_schema"]["name"],
            "identify_response"
        );
        assert!(
            body["response_format"]["json_schema"]["strict"]
                .as_bool()
                .unwrap()
        );

        // Schema wraps results in suggestions array
        let schema = &body["response_format"]["json_schema"]["schema"];
        assert_eq!(schema["properties"]["suggestions"]["type"], "array");
        assert!(schema["properties"]["suggestions"]["items"].is_object());

        // Schema includes rejected fields
        assert_eq!(schema["properties"]["rejected"]["type"], "boolean");
        let required = schema["required"].as_array().unwrap();
        assert!(required.contains(&json!("rejected")));
        assert!(required.contains(&json!("rejected_reason")));

        assert_eq!(body["messages"][0]["role"], "user");

        let parts = body["messages"][0]["content"].as_array().unwrap();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0]["type"], "text");
        assert!(
            parts[0]["text"]
                .as_str()
                .unwrap()
                .contains("top 3 most likely")
        );
        assert_eq!(parts[1]["type"], "image_url");
        assert!(
            parts[1]["image_url"]["url"]
                .as_str()
                .unwrap()
                .starts_with("data:image/jpeg;base64,")
        );
    }

    // --- SSE parsing tests ---

    #[test]
    fn parse_sse_line_extracts_delta() {
        let line = r#"data: {"choices":[{"delta":{"content":"Hello"},"index":0}]}"#;
        assert_eq!(parse_sse_line(line), Some("Hello".to_string()));
    }

    #[test]
    fn parse_sse_line_skips_done_marker() {
        assert_eq!(parse_sse_line("data: [DONE]"), None);
    }

    #[test]
    fn parse_sse_line_skips_empty_lines() {
        assert_eq!(parse_sse_line(""), None);
        assert_eq!(parse_sse_line("\n"), None);
    }

    #[test]
    fn parse_sse_line_skips_non_data_lines() {
        assert_eq!(parse_sse_line("event: message"), None);
        assert_eq!(parse_sse_line(": comment"), None);
    }

    #[test]
    fn parse_sse_line_skips_empty_content() {
        let line = r#"data: {"choices":[{"delta":{"content":""},"index":0}]}"#;
        assert_eq!(parse_sse_line(line), None);
    }

    #[test]
    fn parse_sse_line_skips_role_only_delta() {
        let line = r#"data: {"choices":[{"delta":{"role":"assistant"},"index":0}]}"#;
        assert_eq!(parse_sse_line(line), None);
    }

    #[test]
    fn parse_sse_line_handles_malformed_json() {
        assert_eq!(parse_sse_line("data: {invalid json}"), None);
    }

    // --- Summarize response parsing tests ---

    #[test]
    fn parse_summarize_response_valid() {
        let content = r#"{"summary":"Diagnosed yellowing as overwatering."}"#;
        let parsed: Value = serde_json::from_str(content).unwrap();
        let summary = parsed["summary"].as_str().unwrap();
        assert_eq!(summary, "Diagnosed yellowing as overwatering.");
    }

    #[test]
    fn parse_summarize_response_missing_field() {
        let content = r#"{"note":"Something else"}"#;
        let parsed: Value = serde_json::from_str(content).unwrap();
        assert!(parsed["summary"].as_str().is_none());
    }

    #[test]
    fn parse_summarize_response_invalid_json() {
        let result = serde_json::from_str::<Value>("{not valid}");
        assert!(result.is_err());
    }

    // --- Prompt caching tests ---

    /// OpenAI automatic prompt caching reuses cached prefixes when the beginning
    /// of the messages array is byte-identical across requests.  This test
    /// simulates two consecutive chat turns and asserts that the message prefix
    /// from the first request is preserved exactly in the second request, so
    /// caching can kick in.
    #[test]
    fn chat_messages_prefix_stable_across_turns_for_prompt_caching() {
        let system = "You are a plant care assistant.";

        // --- Turn 1: single user message ---
        let turn1_messages = vec![ChatMessage {
            role: "user".to_string(),
            content: "Why are the leaves yellow?".to_string(),
            image: None,
        }];
        let turn1 = build_chat_messages(system, &turn1_messages, None);

        // --- Turn 2: previous exchange + new user message ---
        let turn2_messages = vec![
            ChatMessage {
                role: "user".to_string(),
                content: "Why are the leaves yellow?".to_string(),
                image: None,
            },
            ChatMessage {
                role: "assistant".to_string(),
                content: "It could be overwatering.".to_string(),
                image: None,
            },
            ChatMessage {
                role: "user".to_string(),
                content: "How do I fix it?".to_string(),
                image: None,
            },
        ];
        let turn2 = build_chat_messages(system, &turn2_messages, None);

        // The first N messages from turn 1 must appear identically as the
        // prefix of turn 2 so OpenAI can cache the shared prefix.
        assert!(
            turn2.len() > turn1.len(),
            "Turn 2 should have more messages than turn 1"
        );
        for (i, msg) in turn1.iter().enumerate() {
            assert_eq!(
                msg, &turn2[i],
                "Message at index {i} differs between turns — breaks prompt caching"
            );
        }

        // Also verify serialized JSON is byte-identical for the shared prefix,
        // which is what OpenAI actually compares.
        let turn1_json = serde_json::to_string(&turn1).unwrap();
        let turn2_prefix_json = serde_json::to_string(&turn2[..turn1.len()]).unwrap();
        assert_eq!(
            turn1_json, turn2_prefix_json,
            "Serialized message prefix must be byte-identical for prompt caching"
        );
    }

    /// Verify that an image attached to a history message does not corrupt the
    /// prefix of subsequent messages (image messages use a content-array format
    /// that must remain stable).
    #[test]
    fn chat_messages_prefix_stable_with_image_in_history() {
        let system = "You are a plant care assistant.";
        let img_b64 = STANDARD.encode(b"fake-image");

        // Turn 1: user sends a message with an image
        let turn1_messages = vec![ChatMessage {
            role: "user".to_string(),
            content: "What is this spot?".to_string(),
            image: Some(img_b64.clone()),
        }];
        let turn1 = build_chat_messages(system, &turn1_messages, None);

        // Turn 2: same history + assistant reply + new question
        let turn2_messages = vec![
            ChatMessage {
                role: "user".to_string(),
                content: "What is this spot?".to_string(),
                image: Some(img_b64),
            },
            ChatMessage {
                role: "assistant".to_string(),
                content: "Looks like sunburn.".to_string(),
                image: None,
            },
            ChatMessage {
                role: "user".to_string(),
                content: "Should I move it?".to_string(),
                image: None,
            },
        ];
        let turn2 = build_chat_messages(system, &turn2_messages, None);

        let turn1_json = serde_json::to_string(&turn1).unwrap();
        let turn2_prefix_json = serde_json::to_string(&turn2[..turn1.len()]).unwrap();
        assert_eq!(
            turn1_json, turn2_prefix_json,
            "Serialized prefix with image history must be byte-identical"
        );
    }
}
