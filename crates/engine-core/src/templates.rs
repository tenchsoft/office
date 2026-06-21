use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

/// A prompt template with variable placeholders like {{variable}}.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PromptTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub template: String,
    pub variables: Vec<String>,
    pub category: String,
    pub product: Option<String>,
}

impl PromptTemplate {
    /// Parse template text to extract {{variable}} placeholders.
    pub fn parse(template: &str) -> Vec<String> {
        let re = regex::Regex::new(r"\{\{(\w+)\}\}").unwrap();
        re.captures_iter(template)
            .filter_map(|c| c.get(1).map(|m| m.as_str().to_string()))
            .collect()
    }

    /// Render the template with the given variable values.
    pub fn render(&self, vars: &HashMap<String, String>) -> Result<String, String> {
        let mut result = self.template.clone();
        for var in &self.variables {
            let placeholder = format!("{{{{{var}}}}}");
            let value = vars
                .get(var)
                .ok_or_else(|| format!("Missing variable: {var}"))?;
            result = result.replace(&placeholder, value);
        }
        Ok(result)
    }
}

/// Registry of prompt templates organized by product and category.
#[derive(Clone, Debug, Default)]
pub struct PromptTemplateRegistry {
    templates: Arc<Mutex<HashMap<String, PromptTemplate>>>,
}

impl PromptTemplateRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&self, template: PromptTemplate) {
        self.templates
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(template.id.clone(), template);
    }

    pub fn get(&self, id: &str) -> Option<PromptTemplate> {
        self.templates
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(id)
            .cloned()
    }

    pub fn list_by_product(&self, product: &str) -> Vec<PromptTemplate> {
        self.templates
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .values()
            .filter(|t| t.product.as_deref() == Some(product))
            .cloned()
            .collect()
    }

    pub fn list_by_category(&self, category: &str) -> Vec<PromptTemplate> {
        self.templates
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .values()
            .filter(|t| t.category == category)
            .cloned()
            .collect()
    }

    pub fn list_all(&self) -> Vec<PromptTemplate> {
        self.templates
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .values()
            .cloned()
            .collect()
    }

    pub fn remove(&self, id: &str) -> bool {
        self.templates
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .remove(id)
            .is_some()
    }
}

/// Built-in template IDs for office products.
#[allow(dead_code)]
pub mod builtin {
    use super::{PromptTemplate, PromptTemplateRegistry};

    pub const DOCS_SUMMARIZE: &str = "docs-summarize";
    pub const DOCS_GRAMMAR: &str = "docs-grammar";
    pub const DOCS_TRANSLATE: &str = "docs-translate";
    pub const DOCS_STYLE: &str = "docs-style";
    pub const DOCS_WRITE: &str = "docs-write";

    pub const SHEETS_FORMULA: &str = "sheets-formula";
    pub const SHEETS_ANALYZE: &str = "sheets-analyze";
    pub const SHEETS_CLEAN: &str = "sheets-clean";

    pub const SLIDES_GENERATE: &str = "slides-generate";
    pub const SLIDES_DESIGN: &str = "slides-design";
    pub const SLIDES_SCRIPT: &str = "slides-script";

    pub fn register_all(registry: &PromptTemplateRegistry) {
        let templates = vec![
            PromptTemplate {
                id: DOCS_SUMMARIZE.to_string(),
                name: "Summarize Document".into(),
                description: "Generate a summary of the document content".into(),
                template: "Summarize the following text concisely:\n\n{{content}}\n\nSummary:".into(),
                variables: vec!["content".into()],
                category: "summarize".into(),
                product: Some("docs".into()),
            },
            PromptTemplate {
                id: DOCS_GRAMMAR.to_string(),
                name: "Grammar Check".into(),
                description: "Check and correct grammar in the text".into(),
                template: "Correct any grammar, spelling, or punctuation errors in the following text. Return only the corrected text:\n\n{{content}}".into(),
                variables: vec!["content".into()],
                category: "grammar".into(),
                product: Some("docs".into()),
            },
            PromptTemplate {
                id: DOCS_TRANSLATE.to_string(),
                name: "Translate".into(),
                description: "Translate text to the target language".into(),
                template: "Translate the following text to {{language}}:\n\n{{content}}".into(),
                variables: vec!["content".into(), "language".into()],
                category: "translate".into(),
                product: Some("docs".into()),
            },
            PromptTemplate {
                id: DOCS_STYLE.to_string(),
                name: "Style Change".into(),
                description: "Change the writing style".into(),
                template: "Rewrite the following text in a {{style}} style:\n\n{{content}}".into(),
                variables: vec!["content".into(), "style".into()],
                category: "style".into(),
                product: Some("docs".into()),
            },
            PromptTemplate {
                id: DOCS_WRITE.to_string(),
                name: "Writing Assist".into(),
                description: "Continue writing from the given context".into(),
                template: "Continue writing the following text naturally:\n\n{{content}}".into(),
                variables: vec!["content".into()],
                category: "write".into(),
                product: Some("docs".into()),
            },
            PromptTemplate {
                id: SHEETS_FORMULA.to_string(),
                name: "Formula Generator".into(),
                description: "Generate a spreadsheet formula".into(),
                template: "Generate a spreadsheet formula for: {{description}}\nContext: {{context}}".into(),
                variables: vec!["description".into(), "context".into()],
                category: "formula".into(),
                product: Some("sheets".into()),
            },
            PromptTemplate {
                id: SHEETS_ANALYZE.to_string(),
                name: "Data Analysis".into(),
                description: "Analyze spreadsheet data".into(),
                template: "Analyze the following data and provide insights:\n\n{{data}}".into(),
                variables: vec!["data".into()],
                category: "analyze".into(),
                product: Some("sheets".into()),
            },
            PromptTemplate {
                id: SHEETS_CLEAN.to_string(),
                name: "Data Cleaning".into(),
                description: "Clean and normalize data".into(),
                template: "Clean and normalize the following data. Fix formatting, remove duplicates, standardize entries:\n\n{{data}}".into(),
                variables: vec!["data".into()],
                category: "clean".into(),
                product: Some("sheets".into()),
            },
            PromptTemplate {
                id: SLIDES_GENERATE.to_string(),
                name: "Slide Generator".into(),
                description: "Generate slides from a topic".into(),
                template: "Create a {{count}}-slide presentation about: {{topic}}\nAudience: {{audience}}\nTone: {{tone}}".into(),
                variables: vec!["count".into(), "topic".into(), "audience".into(), "tone".into()],
                category: "generate".into(),
                product: Some("slides".into()),
            },
            PromptTemplate {
                id: SLIDES_DESIGN.to_string(),
                name: "Design Recommendation".into(),
                description: "Recommend design improvements".into(),
                template: "Suggest design improvements for the following slide content:\n\n{{content}}".into(),
                variables: vec!["content".into()],
                category: "design".into(),
                product: Some("slides".into()),
            },
            PromptTemplate {
                id: SLIDES_SCRIPT.to_string(),
                name: "Presentation Script".into(),
                description: "Generate a presentation script".into(),
                template: "Write a presentation script for the following slides:\n\n{{slides}}".into(),
                variables: vec!["slides".into()],
                category: "script".into(),
                product: Some("slides".into()),
            },
        ];

        for t in templates {
            registry.register(t);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_extracts_variables() {
        let vars = PromptTemplate::parse("Hello {{name}}, {{greeting}}!");
        assert_eq!(vars, vec!["name", "greeting"]);
    }

    #[test]
    fn render_fills_variables() {
        let t = PromptTemplate {
            id: "test".into(),
            name: "Test".into(),
            description: "Test".into(),
            template: "Hello {{name}}!".into(),
            variables: vec!["name".into()],
            category: "test".into(),
            product: None,
        };
        let mut vars = HashMap::new();
        vars.insert("name".into(), "World".into());
        assert_eq!(t.render(&vars).unwrap(), "Hello World!");
    }

    #[test]
    fn render_fails_on_missing_var() {
        let t = PromptTemplate {
            id: "test".into(),
            name: "Test".into(),
            description: "Test".into(),
            template: "{{a}} {{b}}".into(),
            variables: vec!["a".into(), "b".into()],
            category: "test".into(),
            product: None,
        };
        let mut vars = HashMap::new();
        vars.insert("a".into(), "1".into());
        assert!(t.render(&vars).is_err());
    }

    #[test]
    fn builtin_templates_load() {
        let registry = PromptTemplateRegistry::new();
        builtin::register_all(&registry);
        assert!(registry.get(builtin::DOCS_SUMMARIZE).is_some());
        assert_eq!(registry.list_by_product("docs").len(), 5);
        assert_eq!(registry.list_by_product("sheets").len(), 3);
        assert_eq!(registry.list_by_product("slides").len(), 3);
    }
}
