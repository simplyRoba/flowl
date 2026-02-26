use async_trait::async_trait;

use super::types::IdentifyResponse;

#[async_trait]
pub trait AiProvider: Send + Sync {
    async fn identify(
        &self,
        images: &[&[u8]],
        locale: &str,
    ) -> Result<IdentifyResponse, Box<dyn std::error::Error + Send + Sync>>;

    async fn chat(
        &self,
        system_prompt: &str,
        messages: &[super::types::ChatMessage],
        image: Option<&[u8]>,
        locale: &str,
    ) -> Result<super::types::ChatResponseStream, Box<dyn std::error::Error + Send + Sync>>;

    async fn summarize(
        &self,
        system_prompt: &str,
        messages: &[super::types::ChatMessage],
        locale: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;
}
