use serde::{Deserialize, Serialize};

/// Document-level metadata.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct TdmMetadata {
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
}

/// Header and footer content for a document section.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct HeadersFooters {
    #[serde(default)]
    pub default_header: Option<String>,
    #[serde(default)]
    pub default_footer: Option<String>,
    #[serde(default)]
    pub first_page_header: Option<String>,
    #[serde(default)]
    pub first_page_footer: Option<String>,
    #[serde(default)]
    pub even_page_header: Option<String>,
    #[serde(default)]
    pub even_page_footer: Option<String>,
}

impl HeadersFooters {
    /// Resolve auto fields in a header/footer template string.
    ///
    /// Supported fields: `{{page}}`, `{{pages}}`, `{{date}}`, `{{title}}`.
    pub fn resolve_fields(template: &str, page: usize, pages: usize, title: &str) -> String {
        let today = chrono_like_date_string();
        template
            .replace("{{page}}", &page.to_string())
            .replace("{{pages}}", &pages.to_string())
            .replace("{{date}}", &today)
            .replace("{{title}}", title)
    }

    /// Return the appropriate header for the given page number (1-indexed).
    pub fn header_for_page(&self, page: usize, pages: usize, title: &str) -> Option<String> {
        let raw = if page == 1 {
            self.first_page_header
                .as_deref()
                .or(self.default_header.as_deref())?
        } else if page.is_multiple_of(2) {
            self.even_page_header
                .as_deref()
                .or(self.default_header.as_deref())?
        } else {
            self.default_header.as_deref()?
        };
        Some(Self::resolve_fields(raw, page, pages, title))
    }

    /// Return the appropriate footer for the given page number (1-indexed).
    pub fn footer_for_page(&self, page: usize, pages: usize, title: &str) -> Option<String> {
        let raw = if page == 1 {
            self.first_page_footer
                .as_deref()
                .or(self.default_footer.as_deref())?
        } else if page.is_multiple_of(2) {
            self.even_page_footer
                .as_deref()
                .or(self.default_footer.as_deref())?
        } else {
            self.default_footer.as_deref()?
        };
        Some(Self::resolve_fields(raw, page, pages, title))
    }
}

/// Produce a simple date string without depending on chrono.
fn chrono_like_date_string() -> String {
    // Use std::time for a basic ISO-ish date. Since we cannot get the calendar
    // date from SystemTime without chrono, we produce a placeholder that is
    // still useful for rendering. A future integration with chrono or time
    // crate can replace this.
    let duration = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let total_days = duration.as_secs() / 86400;
    // Simple day count from epoch — not calendar-accurate but sufficient for
    // template rendering in the UI. For real calendar dates, integrate chrono.
    format!("day+{}", total_days)
}

/// A named style definition that can be referenced by blocks or inline content.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct StyleDef {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub parent_id: Option<String>,
    #[serde(default)]
    pub is_default: bool,
}
