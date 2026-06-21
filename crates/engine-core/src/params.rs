//! Model Parameter Tuning Interface (#465)
//!
//! Provides a structured way to configure model parameters
//! such as temperature, top_p, frequency/presence penalties, etc.

use serde::{Deserialize, Serialize};

/// Full set of tunable model parameters.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModelParams {
    #[serde(default = "default_temperature")]
    pub temperature: f64,
    #[serde(default = "default_top_p")]
    pub top_p: f64,
    #[serde(default)]
    pub max_tokens: Option<usize>,
    #[serde(default)]
    pub frequency_penalty: Option<f64>,
    #[serde(default)]
    pub presence_penalty: Option<f64>,
    #[serde(default)]
    pub stop_sequences: Vec<String>,
    #[serde(default)]
    pub seed: Option<u64>,
    #[serde(default = "default_true")]
    pub stream: bool,
}

impl Default for ModelParams {
    fn default() -> Self {
        Self {
            temperature: default_temperature(),
            top_p: default_top_p(),
            max_tokens: None,
            frequency_penalty: None,
            presence_penalty: None,
            stop_sequences: Vec::new(),
            seed: None,
            stream: true,
        }
    }
}

fn default_temperature() -> f64 {
    0.7
}
fn default_top_p() -> f64 {
    1.0
}
fn default_true() -> bool {
    true
}

/// Preset parameter profiles for common use cases.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum ParamPreset {
    /// Deterministic, factual output.
    Precise,
    /// Balanced creativity and accuracy.
    Balanced,
    /// High creativity for brainstorming.
    Creative,
    /// Maximum diversity for exploration.
    Exploratory,
    /// Code generation optimized.
    Code,
}

impl ParamPreset {
    pub fn to_params(&self) -> ModelParams {
        match self {
            Self::Precise => ModelParams {
                temperature: 0.1,
                top_p: 0.9,
                frequency_penalty: Some(0.0),
                presence_penalty: Some(0.0),
                ..ModelParams::default()
            },
            Self::Balanced => ModelParams {
                temperature: 0.7,
                top_p: 1.0,
                ..ModelParams::default()
            },
            Self::Creative => ModelParams {
                temperature: 1.0,
                top_p: 0.95,
                frequency_penalty: Some(0.3),
                presence_penalty: Some(0.3),
                ..ModelParams::default()
            },
            Self::Exploratory => ModelParams {
                temperature: 1.5,
                top_p: 0.95,
                frequency_penalty: Some(0.5),
                presence_penalty: Some(0.5),
                ..ModelParams::default()
            },
            Self::Code => ModelParams {
                temperature: 0.2,
                top_p: 0.95,
                max_tokens: Some(4096),
                ..ModelParams::default()
            },
        }
    }

    pub fn label(&self) -> &str {
        match self {
            Self::Precise => "Precise",
            Self::Balanced => "Balanced",
            Self::Creative => "Creative",
            Self::Exploratory => "Exploratory",
            Self::Code => "Code",
        }
    }
}

/// Validates model parameters are within acceptable ranges.
pub fn validate_params(params: &ModelParams) -> Result<(), String> {
    if !(0.0..=2.0).contains(&params.temperature) {
        return Err(format!(
            "temperature must be between 0.0 and 2.0, got {}",
            params.temperature
        ));
    }
    if !(0.0..=1.0).contains(&params.top_p) {
        return Err(format!(
            "top_p must be between 0.0 and 1.0, got {}",
            params.top_p
        ));
    }
    if let Some(fp) = params.frequency_penalty {
        if !(-2.0..=2.0).contains(&fp) {
            return Err(format!(
                "frequency_penalty must be between -2.0 and 2.0, got {}",
                fp
            ));
        }
    }
    if let Some(pp) = params.presence_penalty {
        if !(-2.0..=2.0).contains(&pp) {
            return Err(format!(
                "presence_penalty must be between -2.0 and 2.0, got {}",
                pp
            ));
        }
    }
    if let Some(mt) = params.max_tokens {
        if mt == 0 {
            return Err("max_tokens must be > 0".into());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preset_precise_has_low_temp() {
        let params = ParamPreset::Precise.to_params();
        assert!((params.temperature - 0.1).abs() < f64::EPSILON);
    }

    #[test]
    fn preset_code_has_max_tokens() {
        let params = ParamPreset::Code.to_params();
        assert_eq!(params.max_tokens, Some(4096));
    }

    #[test]
    fn validate_rejects_bad_temperature() {
        let params = ModelParams {
            temperature: 5.0,
            ..ModelParams::default()
        };
        assert!(validate_params(&params).is_err());
    }

    #[test]
    fn validate_accepts_defaults() {
        let params = ModelParams::default();
        assert!(validate_params(&params).is_ok());
    }
}
