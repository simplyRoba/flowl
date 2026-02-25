use async_trait::async_trait;

use super::types::IdentifyResult;

#[async_trait]
pub trait AiProvider: Send + Sync {
    async fn identify(
        &self,
        images: &[&[u8]],
        locale: &str,
    ) -> Result<IdentifyResult, Box<dyn std::error::Error + Send + Sync>>;

    async fn chat(
        &self,
        messages: &[super::types::ChatMessage],
    ) -> Result<super::types::ChatResponseStream, Box<dyn std::error::Error + Send + Sync>>;

    async fn summarize(
        &self,
        text: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>>;
}
