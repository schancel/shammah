// Fallback chain for automatic provider retry
//
// Tries providers in priority order until one succeeds

use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::sync::mpsc;

use super::{LlmProvider, ProviderRequest, ProviderResponse, StreamChunk};

/// A chain of providers to try in order
pub struct FallbackChain {
    providers: Arc<Vec<Box<dyn LlmProvider>>>,
}

impl FallbackChain {
    /// Create a new fallback chain with providers in priority order
    pub fn new(providers: Vec<Box<dyn LlmProvider>>) -> Self {
        Self {
            providers: Arc::new(providers),
        }
    }

    /// Get the number of providers in the chain
    pub fn len(&self) -> usize {
        self.providers.len()
    }

    /// Check if the chain is empty
    pub fn is_empty(&self) -> bool {
        self.providers.is_empty()
    }

    /// Get the primary provider (first in chain)
    pub fn primary_provider(&self) -> Option<&dyn LlmProvider> {
        self.providers.first().map(|p| p.as_ref())
    }

    /// Try sending message with automatic fallback
    pub async fn send_message_with_fallback(
        &self,
        request: &ProviderRequest,
    ) -> Result<ProviderResponse> {
        let mut last_error = None;

        for (idx, provider) in self.providers.iter().enumerate() {
            tracing::info!(
                "Trying provider {} ({}/{})",
                provider.name(),
                idx + 1,
                self.providers.len()
            );

            // Create a modified request with this provider's model ID
            let provider_request = ProviderRequest {
                model: provider.default_model().to_string(),
                messages: request.messages.clone(),
                max_tokens: request.max_tokens,
                tools: request.tools.clone(),
                temperature: request.temperature,
                stream: request.stream,
            };

            match provider.send_message(&provider_request).await {
                Ok(response) => {
                    if idx > 0 {
                        tracing::info!(
                            "Provider {} succeeded after {} failed attempts",
                            provider.name(),
                            idx
                        );
                    } else {
                        tracing::debug!("Primary provider {} succeeded", provider.name());
                    }
                    return Ok(response);
                }
                Err(e) => {
                    tracing::warn!(
                        "Provider {} failed (attempt {}/{}): {}",
                        provider.name(),
                        idx + 1,
                        self.providers.len(),
                        e
                    );
                    last_error = Some(e);
                    continue;
                }
            }
        }

        Err(last_error
            .unwrap_or_else(|| anyhow::anyhow!("No providers available"))
            .context("All fallback providers failed"))
    }

    /// Try streaming with automatic fallback, including mid-stream error recovery
    ///
    /// This method wraps provider receivers to handle errors that occur during streaming.
    /// If a provider's receiver sends an error mid-stream, it automatically retries
    /// with the next provider in the chain.
    pub async fn send_message_stream_with_fallback(
        &self,
        request: &ProviderRequest,
    ) -> Result<mpsc::Receiver<Result<StreamChunk>>> {
        // Create a channel for the wrapped receiver
        let (tx, rx) = mpsc::channel(100);

        // Clone data for the spawned task
        let providers = Arc::clone(&self.providers);
        let request = request.clone();

        // Spawn task to try providers in sequence with automatic retry on stream errors
        tokio::spawn(async move {
            let mut provider_idx = 0;

            while provider_idx < providers.len() {
                let provider = &providers[provider_idx];
                tracing::info!(
                    "Trying streaming with provider {} ({}/{})",
                    provider.name(),
                    provider_idx + 1,
                    providers.len()
                );

                // Create a modified request with this provider's model ID
                let provider_request = ProviderRequest {
                    model: provider.default_model().to_string(),
                    messages: request.messages.clone(),
                    max_tokens: request.max_tokens,
                    tools: request.tools.clone(),
                    temperature: request.temperature,
                    stream: request.stream,
                };

                match provider.send_message_stream(&provider_request).await {
                    Ok(mut receiver) => {
                        if provider_idx > 0 {
                            tracing::info!(
                                "Provider {} streaming succeeded after {} failed attempts",
                                provider.name(),
                                provider_idx
                            );
                        } else {
                            tracing::debug!("Primary provider {} streaming succeeded", provider.name());
                        }

                        // Forward chunks from this provider's receiver
                        let mut had_error = false;
                        while let Some(result) = receiver.recv().await {
                            match result {
                                Ok(chunk) => {
                                    if tx.send(Ok(chunk)).await.is_err() {
                                        // Receiver dropped, stop streaming
                                        tracing::debug!("Receiver dropped, stopping stream");
                                        return;
                                    }
                                }
                                Err(e) => {
                                    // Stream error - try next provider if available
                                    tracing::warn!(
                                        "Provider {} stream error: {}. Trying fallback...",
                                        provider.name(),
                                        e
                                    );
                                    had_error = true;
                                    break;
                                }
                            }
                        }

                        if !had_error {
                            // Stream completed successfully
                            tracing::debug!("Provider {} stream completed successfully", provider.name());
                            return;
                        }

                        // Had error, try next provider
                        provider_idx += 1;
                    }
                    Err(e) => {
                        tracing::warn!(
                            "Provider {} streaming failed (attempt {}/{}): {}",
                            provider.name(),
                            provider_idx + 1,
                            providers.len(),
                            e
                        );
                        provider_idx += 1;
                        continue;
                    }
                }
            }

            // All providers failed
            let error_msg = "All fallback providers failed for streaming";
            tracing::error!("{}", error_msg);
            let _ = tx.send(Err(anyhow::anyhow!(error_msg))).await;
        });

        Ok(rx)
    }
}

// Implement LlmProvider trait for FallbackChain
#[async_trait::async_trait]
impl LlmProvider for FallbackChain {
    async fn send_message(&self, request: &ProviderRequest) -> Result<ProviderResponse> {
        self.send_message_with_fallback(request).await
    }

    async fn send_message_stream(
        &self,
        request: &ProviderRequest,
    ) -> Result<mpsc::Receiver<Result<StreamChunk>>> {
        self.send_message_stream_with_fallback(request).await
    }

    fn name(&self) -> &str {
        self.primary_provider()
            .map(|p| p.name())
            .unwrap_or("FallbackChain")
    }

    fn default_model(&self) -> &str {
        self.primary_provider()
            .map(|p| p.default_model())
            .unwrap_or("default")
    }

    fn supports_streaming(&self) -> bool {
        self.primary_provider()
            .map(|p| p.supports_streaming())
            .unwrap_or(false)
    }

    fn supports_tools(&self) -> bool {
        self.primary_provider()
            .map(|p| p.supports_tools())
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::claude::types::ContentBlock;

    // Mock provider for testing
    struct MockProvider {
        name: String,
        should_fail: bool,
    }

    impl MockProvider {
        fn new(name: &str, should_fail: bool) -> Self {
            Self {
                name: name.to_string(),
                should_fail,
            }
        }
    }

    #[async_trait::async_trait]
    impl LlmProvider for MockProvider {
        async fn send_message(&self, _request: &ProviderRequest) -> Result<ProviderResponse> {
            if self.should_fail {
                anyhow::bail!("Mock provider {} failed", self.name);
            }

            Ok(ProviderResponse {
                id: "test-id".to_string(),
                model: "test-model".to_string(),
                content: vec![ContentBlock::Text {
                    text: "Test response".to_string(),
                }],
                stop_reason: Some("end_turn".to_string()),
                role: "assistant".to_string(),
                provider: self.name.clone(),
            })
        }

        async fn send_message_stream(
            &self,
            _request: &ProviderRequest,
        ) -> Result<mpsc::Receiver<Result<StreamChunk>>> {
            if self.should_fail {
                anyhow::bail!("Mock provider {} streaming failed", self.name);
            }

            let (tx, rx) = mpsc::channel(1);
            tokio::spawn(async move {
                let _ = tx.send(Ok(StreamChunk::TextDelta("test".to_string()))).await;
            });
            Ok(rx)
        }

        fn name(&self) -> &str {
            &self.name
        }

        fn default_model(&self) -> &str {
            "test-model"
        }

        fn supports_streaming(&self) -> bool {
            true
        }

        fn supports_tools(&self) -> bool {
            true
        }
    }

    #[tokio::test]
    async fn test_primary_provider_succeeds() {
        let providers: Vec<Box<dyn LlmProvider>> = vec![
            Box::new(MockProvider::new("primary", false)),
            Box::new(MockProvider::new("fallback", false)),
        ];

        let chain = FallbackChain::new(providers);
        let request = ProviderRequest {
            messages: vec![],
            model: String::new(),
            max_tokens: 100,
            temperature: None,
            tools: None,
            stream: false,
        };

        let result = chain.send_message_with_fallback(&request).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().provider, "primary");
    }

    #[tokio::test]
    async fn test_fallback_to_secondary() {
        let providers: Vec<Box<dyn LlmProvider>> = vec![
            Box::new(MockProvider::new("primary", true)),
            Box::new(MockProvider::new("fallback", false)),
        ];

        let chain = FallbackChain::new(providers);
        let request = ProviderRequest {
            messages: vec![],
            model: String::new(),
            max_tokens: 100,
            temperature: None,
            tools: None,
            stream: false,
        };

        let result = chain.send_message_with_fallback(&request).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().provider, "fallback");
    }

    #[tokio::test]
    async fn test_all_providers_fail() {
        let providers: Vec<Box<dyn LlmProvider>> = vec![
            Box::new(MockProvider::new("primary", true)),
            Box::new(MockProvider::new("fallback", true)),
        ];

        let chain = FallbackChain::new(providers);
        let request = ProviderRequest {
            messages: vec![],
            model: String::new(),
            max_tokens: 100,
            temperature: None,
            tools: None,
            stream: false,
        };

        let result = chain.send_message_with_fallback(&request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_streaming_fallback() {
        let providers: Vec<Box<dyn LlmProvider>> = vec![
            Box::new(MockProvider::new("primary", true)),
            Box::new(MockProvider::new("fallback", false)),
        ];

        let chain = FallbackChain::new(providers);
        let request = ProviderRequest {
            messages: vec![],
            model: String::new(),
            max_tokens: 100,
            temperature: None,
            tools: None,
            stream: true,
        };

        let result = chain.send_message_stream_with_fallback(&request).await;
        assert!(result.is_ok());
    }
}
