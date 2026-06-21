//! Per-Product Model Configuration (#464)
//!
//! Each product (Docs, Sheets, Slides, etc.) can have its own
//! default model and parameter overrides.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Product identifier.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum ProductId {
    Docs,
    Sheets,
    Slides,
    Research,
    Story,
    Code,
    Universe,
    Study,
}

impl ProductId {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Docs => "docs",
            Self::Sheets => "sheets",
            Self::Slides => "slides",
            Self::Research => "research",
            Self::Story => "story",
            Self::Code => "code",
            Self::Universe => "universe",
            Self::Study => "study",
        }
    }

    pub fn from_str_lossy(s: &str) -> Self {
        match s {
            "docs" => Self::Docs,
            "sheets" => Self::Sheets,
            "slides" => Self::Slides,
            "research" => Self::Research,
            "story" => Self::Story,
            "code" => Self::Code,
            "universe" => Self::Universe,
            "study" => Self::Study,
            _ => Self::Docs,
        }
    }

    pub fn all() -> &'static [ProductId] {
        &[
            Self::Docs,
            Self::Sheets,
            Self::Slides,
            Self::Research,
            Self::Story,
            Self::Code,
            Self::Universe,
            Self::Study,
        ]
    }
}

/// Model parameters that a product can override.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProductModelConfig {
    /// Default model ID for this product.
    pub model_id: Option<String>,
    /// Default provider for this product.
    pub provider: Option<String>,
    /// Temperature override (0.0 - 2.0).
    pub temperature: Option<f64>,
    /// Max tokens override.
    pub max_tokens: Option<usize>,
    /// System prompt prefix for this product.
    pub system_prompt_prefix: Option<String>,
}

/// Manages per-product model configurations.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProductConfig {
    /// Global default model ID.
    pub default_model_id: String,
    /// Global default provider.
    pub default_provider: String,
    /// Per-product overrides.
    pub products: HashMap<String, ProductModelConfig>,
}

impl ProductConfig {
    pub fn new(default_model_id: String, default_provider: String) -> Self {
        Self {
            default_model_id,
            default_provider,
            products: HashMap::new(),
        }
    }

    /// Set a product's model configuration.
    pub fn set(&mut self, product: ProductId, config: ProductModelConfig) {
        self.products.insert(product.as_str().to_string(), config);
    }

    /// Get the effective model ID for a product.
    pub fn model_id(&self, product: ProductId) -> &str {
        self.products
            .get(product.as_str())
            .and_then(|c| c.model_id.as_deref())
            .unwrap_or(&self.default_model_id)
    }

    /// Get the effective provider for a product.
    pub fn provider(&self, product: ProductId) -> &str {
        self.products
            .get(product.as_str())
            .and_then(|c| c.provider.as_deref())
            .unwrap_or(&self.default_provider)
    }

    /// Get the effective temperature for a product.
    pub fn temperature(&self, product: ProductId) -> Option<f64> {
        self.products
            .get(product.as_str())
            .and_then(|c| c.temperature)
    }

    /// Get the effective max tokens for a product.
    pub fn max_tokens(&self, product: ProductId) -> Option<usize> {
        self.products
            .get(product.as_str())
            .and_then(|c| c.max_tokens)
    }

    /// Get the system prompt prefix for a product.
    pub fn system_prompt_prefix(&self, product: ProductId) -> Option<&str> {
        self.products
            .get(product.as_str())
            .and_then(|c| c.system_prompt_prefix.as_deref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn product_config_overrides() {
        let mut config = ProductConfig::new("default-model".into(), "mock".into());
        config.set(
            ProductId::Docs,
            ProductModelConfig {
                model_id: Some("gpt-4".into()),
                provider: Some("openai".into()),
                temperature: Some(0.7),
                max_tokens: None,
                system_prompt_prefix: Some("You are a writing assistant.".into()),
            },
        );

        assert_eq!(config.model_id(ProductId::Docs), "gpt-4");
        assert_eq!(config.model_id(ProductId::Sheets), "default-model");
        assert_eq!(config.provider(ProductId::Docs), "openai");
        assert_eq!(config.temperature(ProductId::Docs), Some(0.7));
        assert_eq!(config.temperature(ProductId::Sheets), None);
        assert_eq!(
            config.system_prompt_prefix(ProductId::Docs),
            Some("You are a writing assistant.")
        );
    }
}
