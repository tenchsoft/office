use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Approximate token count using character-based heuristic.
/// In production, this would use a BPE tokenizer (cl100k_base / o200k_base).
#[derive(Clone, Debug)]
pub struct TokenCounter {
    chars_per_token: f64,
}

impl TokenCounter {
    pub fn new() -> Self {
        // ~4 chars per token for English, ~2 for CJK
        Self {
            chars_per_token: 3.5,
        }
    }

    pub fn count(&self, text: &str) -> u32 {
        if text.is_empty() {
            return 0;
        }
        let char_count = text.chars().count() as f64;
        // Adjust for CJK characters (roughly 2 chars per token)
        let cjk_count = text.chars().filter(|c| is_cjk(*c)).count() as f64;
        let latin_count = char_count - cjk_count;
        let tokens = (latin_count / self.chars_per_token) + (cjk_count / 2.0);
        tokens.ceil() as u32
    }

    pub fn count_messages(&self, messages: &[(String, String)]) -> u32 {
        messages
            .iter()
            .map(|(_role, content)| {
                // Role overhead ~4 tokens per message
                4 + self.count(content)
            })
            .sum()
    }
}

fn is_cjk(c: char) -> bool {
    matches!(c,
        '\u{4E00}'..='\u{9FFF}' |   // CJK Unified Ideographs
        '\u{3040}'..='\u{309F}' |   // Hiragana
        '\u{30A0}'..='\u{30FF}' |   // Katakana
        '\u{AC00}'..='\u{D7AF}'     // Hangul Syllables
    )
}

impl Default for TokenCounter {
    fn default() -> Self {
        Self::new()
    }
}

/// Per-model pricing information.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModelPricing {
    pub input_per_million: f64,
    pub output_per_million: f64,
}

/// Cost tracking for API usage.
#[derive(Clone, Debug, Default)]
pub struct CostTracker {
    entries: Arc<Mutex<Vec<CostEntry>>>,
    pricing: Arc<Mutex<HashMap<String, ModelPricing>>>,
}

#[derive(Clone, Debug, Serialize)]
pub struct CostEntry {
    pub model: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub cost_usd: f64,
    pub timestamp_secs: u64,
}

#[derive(Clone, Debug, Serialize)]
pub struct CostSummary {
    pub total_entries: usize,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_cost_usd: f64,
    pub by_model: HashMap<String, ModelCostSummary>,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct ModelCostSummary {
    pub entries: usize,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cost_usd: f64,
}

impl CostTracker {
    pub fn new() -> Self {
        let mut pricing = HashMap::new();
        pricing.insert(
            "tench/chat".to_string(),
            ModelPricing {
                input_per_million: 0.0,
                output_per_million: 0.0,
            },
        );
        pricing.insert(
            "openai/gpt-4o".to_string(),
            ModelPricing {
                input_per_million: 2.50,
                output_per_million: 10.0,
            },
        );
        pricing.insert(
            "anthropic/claude-sonnet-4".to_string(),
            ModelPricing {
                input_per_million: 3.0,
                output_per_million: 15.0,
            },
        );
        pricing.insert(
            "google/gemini-2.0-flash".to_string(),
            ModelPricing {
                input_per_million: 0.10,
                output_per_million: 0.40,
            },
        );
        Self {
            entries: Arc::new(Mutex::new(Vec::new())),
            pricing: Arc::new(Mutex::new(pricing)),
        }
    }

    pub fn record(&self, model: &str, input_tokens: u32, output_tokens: u32) {
        let pricing = self
            .pricing
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let price = pricing.get(model).cloned().unwrap_or(ModelPricing {
            input_per_million: 0.0,
            output_per_million: 0.0,
        });
        let cost = (input_tokens as f64 * price.input_per_million / 1_000_000.0)
            + (output_tokens as f64 * price.output_per_million / 1_000_000.0);
        let timestamp_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let entry = CostEntry {
            model: model.to_string(),
            input_tokens,
            output_tokens,
            cost_usd: cost,
            timestamp_secs,
        };
        self.entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(entry);
    }

    pub fn summary(&self) -> CostSummary {
        let entries = self
            .entries
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let mut total_input = 0u64;
        let mut total_output = 0u64;
        let mut total_cost = 0.0f64;
        let mut by_model: HashMap<String, ModelCostSummary> = HashMap::new();

        for entry in entries.iter() {
            total_input += entry.input_tokens as u64;
            total_output += entry.output_tokens as u64;
            total_cost += entry.cost_usd;
            let model_summary = by_model.entry(entry.model.clone()).or_default();
            model_summary.entries += 1;
            model_summary.input_tokens += entry.input_tokens as u64;
            model_summary.output_tokens += entry.output_tokens as u64;
            model_summary.cost_usd += entry.cost_usd;
        }

        CostSummary {
            total_entries: entries.len(),
            total_input_tokens: total_input,
            total_output_tokens: total_output,
            total_cost_usd: total_cost,
            by_model,
        }
    }

    pub fn set_pricing(&self, model: String, pricing: ModelPricing) {
        self.pricing
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(model, pricing);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_counter_counts_english() {
        let counter = TokenCounter::new();
        let count = counter.count("Hello, world!");
        assert!(count > 0);
        assert!(count < 20);
    }

    #[test]
    fn token_counter_counts_cjk() {
        let counter = TokenCounter::new();
        let count = counter.count("안녕하세요");
        assert!(count > 0);
        assert!(count <= 5);
    }

    #[test]
    fn cost_tracker_records_and_summarizes() {
        let tracker = CostTracker::new();
        tracker.record("openai/gpt-4o", 1000, 500);
        tracker.record("openai/gpt-4o", 2000, 1000);

        let summary = tracker.summary();
        assert_eq!(summary.total_entries, 2);
        assert_eq!(summary.total_input_tokens, 3000);
        assert_eq!(summary.total_output_tokens, 1500);
        assert!(summary.total_cost_usd > 0.0);
    }
}
