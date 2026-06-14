use crate::config::{ModelConfig, ProviderKind};
use async_trait::async_trait;
use rig_core::{client::CompletionClient, completion::Prompt, providers::openai};

#[async_trait]
pub trait AiProvider: Send + Sync {
    async fn complete(&self, model: &ModelConfig, prompt: &str) -> anyhow::Result<String>;
}

#[derive(Debug, Default)]
pub struct RigAiProvider;

#[async_trait]
impl AiProvider for RigAiProvider {
    async fn complete(&self, model: &ModelConfig, prompt: &str) -> anyhow::Result<String> {
        match model.provider {
            ProviderKind::OpenAiCompatible => {
                let api_key = std::env::var(&model.api_key_env)?;
                let mut builder = openai::Client::builder().api_key(&api_key);
                if let Ok(base_url) = std::env::var(&model.endpoint_env)
                    && !base_url.trim().is_empty()
                {
                    builder = builder.base_url(&base_url);
                }
                let client = builder.build()?;
                let agent = client.agent(model.model.clone()).build();
                Ok(agent.prompt(prompt).await?.trim().to_string())
            }
            ProviderKind::Mock => Ok("Mock Title".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn mock_provider_returns_stable_title() {
        let provider = RigAiProvider;
        let model = ModelConfig {
            provider: ProviderKind::Mock,
            ..ModelConfig::default()
        };
        assert_eq!(provider.complete(&model, "x").await.unwrap(), "Mock Title");
    }
}
