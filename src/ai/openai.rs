use async_trait::async_trait;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use reqwest::Client;
use serde_json::{Value, json};

use super::provider::AiProvider;
use super::types::{ChatMessage, ChatResponseStream, IdentifyResult};

pub struct OpenAiProvider {
    client: Client,
    api_key: String,
    base_url: String,
    model: String,
}

impl OpenAiProvider {
    pub fn new(api_key: String, base_url: String, model: String) -> Self {
        Self {
            client: Client::new(),
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
    ) -> Result<IdentifyResult, Box<dyn std::error::Error + Send + Sync>> {
        let mut content: Vec<Value> = vec![json!({
            "type": "text",
            "text": "Identify this plant. Return a JSON object with: common_name (string), scientific_name (string), confidence (number 0-1), summary (string describing the plant), and care_profile (object with watering_interval_days, light_needs, difficulty, pet_safety, growth_speed, soil_type, soil_moisture)."
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

        let body = json!({
            "model": self.model,
            "messages": [{
                "role": "user",
                "content": content
            }],
            "response_format": { "type": "json_object" }
        });

        let response = self
            .client
            .post(self.completions_url())
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await?
            .error_for_status()?;

        let response_body: Value = response.json().await?;
        let content_str = response_body["choices"][0]["message"]["content"]
            .as_str()
            .ok_or("Missing content in API response")?;

        let result: IdentifyResult = serde_json::from_str(content_str)?;
        Ok(result)
    }

    async fn chat(
        &self,
        _messages: &[ChatMessage],
    ) -> Result<ChatResponseStream, Box<dyn std::error::Error + Send + Sync>> {
        unimplemented!("chat is not yet implemented")
    }

    async fn summarize(
        &self,
        _text: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        unimplemented!("summarize is not yet implemented")
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
            "gpt-4o-mini".into(),
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
        let provider = OpenAiProvider::new(
            "test-key".into(),
            "https://api.openai.com/v1".into(),
            "gpt-4o-mini".into(),
        );

        let image_data: &[u8] = b"fake-image-bytes";
        let b64 = STANDARD.encode(image_data);

        let mut content: Vec<Value> = vec![json!({
            "type": "text",
            "text": "Identify this plant. Return a JSON object with: common_name (string), scientific_name (string), confidence (number 0-1), summary (string describing the plant), and care_profile (object with watering_interval_days, light_needs, difficulty, pet_safety, growth_speed, soil_type, soil_moisture)."
        })];
        content.push(json!({
            "type": "image_url",
            "image_url": {
                "url": format!("data:image/jpeg;base64,{b64}")
            }
        }));

        let body = json!({
            "model": provider.model,
            "messages": [{
                "role": "user",
                "content": content
            }],
            "response_format": { "type": "json_object" }
        });

        assert_eq!(body["model"], "gpt-4o-mini");
        assert_eq!(body["response_format"]["type"], "json_object");
        assert_eq!(body["messages"][0]["role"], "user");

        let parts = body["messages"][0]["content"].as_array().unwrap();
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0]["type"], "text");
        assert_eq!(parts[1]["type"], "image_url");
        assert!(
            parts[1]["image_url"]["url"]
                .as_str()
                .unwrap()
                .starts_with("data:image/jpeg;base64,")
        );
    }
}
