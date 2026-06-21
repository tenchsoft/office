use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::tdm::TenchDocument;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OfficeProductKind {
    Docs,
    Sheets,
    Slides,
}

impl OfficeProductKind {
    pub fn product_id(self) -> &'static str {
        match self {
            OfficeProductKind::Docs => "tench-docs",
            OfficeProductKind::Sheets => "tench-sheets",
            OfficeProductKind::Slides => "tench-slides",
        }
    }

    pub fn supports_format(self, format: OfficeFileFormat) -> bool {
        supported_office_formats(self).contains(&format)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OfficeFileFormat {
    Docx,
    Txt,
    Md,
    Html,
    Odt,
    Xlsx,
    Csv,
    Tsv,
    Ods,
    Pptx,
    Odp,
    Pdf,
    Hwp,
    Hwpx,
    Hwt,
    Rtf,
    Epub,
    Images,
    Video,
}

impl OfficeFileFormat {
    pub fn extension(self) -> &'static str {
        match self {
            OfficeFileFormat::Docx => "docx",
            OfficeFileFormat::Txt => "txt",
            OfficeFileFormat::Md => "md",
            OfficeFileFormat::Html => "html",
            OfficeFileFormat::Odt => "odt",
            OfficeFileFormat::Xlsx => "xlsx",
            OfficeFileFormat::Csv => "csv",
            OfficeFileFormat::Tsv => "tsv",
            OfficeFileFormat::Ods => "ods",
            OfficeFileFormat::Pptx => "pptx",
            OfficeFileFormat::Odp => "odp",
            OfficeFileFormat::Pdf => "pdf",
            OfficeFileFormat::Hwp => "hwp",
            OfficeFileFormat::Hwpx => "hwpx",
            OfficeFileFormat::Hwt => "hwt",
            OfficeFileFormat::Rtf => "rtf",
            OfficeFileFormat::Epub => "epub",
            OfficeFileFormat::Images => "images",
            OfficeFileFormat::Video => "video",
        }
    }

    pub fn from_extension(extension: impl AsRef<str>) -> Option<Self> {
        let normalized = extension
            .as_ref()
            .trim()
            .trim_start_matches('.')
            .to_ascii_lowercase();

        match normalized.as_str() {
            "docx" => Some(OfficeFileFormat::Docx),
            "txt" => Some(OfficeFileFormat::Txt),
            "md" | "markdown" => Some(OfficeFileFormat::Md),
            "html" | "htm" => Some(OfficeFileFormat::Html),
            "odt" => Some(OfficeFileFormat::Odt),
            "xlsx" => Some(OfficeFileFormat::Xlsx),
            "csv" => Some(OfficeFileFormat::Csv),
            "tsv" => Some(OfficeFileFormat::Tsv),
            "ods" => Some(OfficeFileFormat::Ods),
            "pptx" => Some(OfficeFileFormat::Pptx),
            "odp" => Some(OfficeFileFormat::Odp),
            "pdf" => Some(OfficeFileFormat::Pdf),
            "hwp" => Some(OfficeFileFormat::Hwp),
            "hwpx" => Some(OfficeFileFormat::Hwpx),
            "hwt" => Some(OfficeFileFormat::Hwt),
            "rtf" => Some(OfficeFileFormat::Rtf),
            "epub" => Some(OfficeFileFormat::Epub),
            "png" | "jpg" | "jpeg" | "webp" | "gif" | "images" => Some(OfficeFileFormat::Images),
            "mp4" | "webm" | "mov" | "video" => Some(OfficeFileFormat::Video),
            _ => None,
        }
    }
}

const DOCS_SUPPORTED_FORMATS: &[OfficeFileFormat] = &[
    OfficeFileFormat::Docx,
    OfficeFileFormat::Txt,
    OfficeFileFormat::Md,
    OfficeFileFormat::Html,
    OfficeFileFormat::Odt,
    OfficeFileFormat::Pdf,
    OfficeFileFormat::Rtf,
    OfficeFileFormat::Epub,
];

const SHEETS_SUPPORTED_FORMATS: &[OfficeFileFormat] = &[
    OfficeFileFormat::Xlsx,
    OfficeFileFormat::Csv,
    OfficeFileFormat::Tsv,
    OfficeFileFormat::Ods,
    OfficeFileFormat::Pdf,
    OfficeFileFormat::Html,
];

const SLIDES_SUPPORTED_FORMATS: &[OfficeFileFormat] = &[
    OfficeFileFormat::Pptx,
    OfficeFileFormat::Odp,
    OfficeFileFormat::Pdf,
    OfficeFileFormat::Images,
    OfficeFileFormat::Video,
];

pub fn supported_office_formats(product: OfficeProductKind) -> &'static [OfficeFileFormat] {
    match product {
        OfficeProductKind::Docs => DOCS_SUPPORTED_FORMATS,
        OfficeProductKind::Sheets => SHEETS_SUPPORTED_FORMATS,
        OfficeProductKind::Slides => SLIDES_SUPPORTED_FORMATS,
    }
}

pub fn detect_office_file_format(path_or_extension: impl AsRef<str>) -> Option<OfficeFileFormat> {
    let value = path_or_extension.as_ref().trim();
    let extension = value
        .rsplit_once('.')
        .map_or(value, |(_, extension)| extension);
    OfficeFileFormat::from_extension(extension)
}

pub fn primary_office_product_for_format(format: OfficeFileFormat) -> Option<OfficeProductKind> {
    match format {
        OfficeFileFormat::Docx
        | OfficeFileFormat::Txt
        | OfficeFileFormat::Md
        | OfficeFileFormat::Html
        | OfficeFileFormat::Odt
        | OfficeFileFormat::Rtf
        | OfficeFileFormat::Epub => Some(OfficeProductKind::Docs),
        OfficeFileFormat::Xlsx
        | OfficeFileFormat::Csv
        | OfficeFileFormat::Tsv
        | OfficeFileFormat::Ods => Some(OfficeProductKind::Sheets),
        OfficeFileFormat::Pptx
        | OfficeFileFormat::Odp
        | OfficeFileFormat::Images
        | OfficeFileFormat::Video => Some(OfficeProductKind::Slides),
        OfficeFileFormat::Hwp
        | OfficeFileFormat::Hwpx
        | OfficeFileFormat::Hwt
        | OfficeFileFormat::Pdf => None,
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficeArtifact {
    pub id: String,
    pub title: String,
    pub product: OfficeProductKind,
    pub format: OfficeFileFormat,
    pub path: Option<String>,
    pub schema_version: String,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    #[serde(default)]
    pub dirty: bool,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub assets: Vec<OfficeAssetRef>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "content", rename_all = "snake_case")]
#[allow(clippy::large_enum_variant)] // Docs variant is larger; boxing would change the public API
pub enum OfficeContent {
    Docs(RichDocumentContent),
    Sheets(WorkbookContent),
    Slides(PresentationContent),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RichDocumentContent {
    pub schema: String,
    /// Tench Document Model representation.
    #[serde(default)]
    pub document: Option<TenchDocument>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WorkbookContent {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub sheets: Vec<SheetContent>,
    pub active_sheet_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SheetContent {
    pub id: String,
    pub name: String,
    pub index: u32,
    #[serde(default)]
    pub cells: Vec<CellContent>,
    pub row_count: Option<u32>,
    pub column_count: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CellContent {
    pub address: String,
    pub value: CellValue,
    pub formula: Option<String>,
    #[serde(default = "empty_json_object")]
    pub style: Value,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum CellValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Empty,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PresentationContent {
    pub id: String,
    pub title: String,
    pub width: u32,
    pub height: u32,
    #[serde(default)]
    pub slides: Vec<SlideContent>,
    #[serde(default)]
    pub assets: Vec<OfficeAssetRef>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SlideContent {
    pub id: String,
    pub index: u32,
    pub title: Option<String>,
    pub notes: Option<String>,
    #[serde(default)]
    pub objects: Vec<SlideObject>,
    #[serde(default = "empty_json_object")]
    pub background: Value,
    #[serde(default = "empty_json_object")]
    pub transition: Value,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SlideObject {
    pub id: String,
    pub object_type: String,
    pub bounds: OfficeRect,
    #[serde(default = "empty_json_object")]
    pub data: Value,
    #[serde(default = "empty_json_object")]
    pub style: Value,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficeRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OfficeAssetKind {
    Image,
    Video,
    Audio,
    Font,
    Thumbnail,
    GeneratedImage,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficeAssetRef {
    pub id: String,
    pub kind: OfficeAssetKind,
    pub path: Option<String>,
    pub mime_type: Option<String>,
    pub checksum: Option<String>,
    #[serde(default = "empty_json_object")]
    pub metadata: Value,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OfficeCreateRequest {
    pub product: OfficeProductKind,
    pub title: String,
    pub template_id: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OfficeOpenRequest {
    pub path: String,
    pub product_hint: Option<OfficeProductKind>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficeOpenResponse {
    pub artifact: OfficeArtifact,
    pub content: OfficeContent,
    #[serde(default)]
    pub diagnostics: Vec<ImportExportDiagnostic>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficeSaveRequest {
    pub artifact: OfficeArtifact,
    pub content: OfficeContent,
    pub options: OfficeSaveOptions,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OfficeSaveOptions {
    pub target_path: Option<String>,
    pub format: Option<OfficeFileFormat>,
    #[serde(default = "default_true")]
    pub atomic: bool,
    #[serde(default = "default_true")]
    pub create_backup: bool,
    #[serde(default = "default_true")]
    pub update_recent: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficeSaveResponse {
    pub artifact: OfficeArtifact,
    pub backup: Option<OfficeBackupMetadata>,
    #[serde(default)]
    pub diagnostics: Vec<ImportExportDiagnostic>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OfficeImportRequest {
    pub source_path: String,
    pub target_product: OfficeProductKind,
    pub source_format: Option<OfficeFileFormat>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficeImportResponse {
    pub artifact: OfficeArtifact,
    pub content: OfficeContent,
    #[serde(default)]
    pub diagnostics: Vec<ImportExportDiagnostic>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficeExportRequest {
    pub artifact_id: String,
    pub source_path: Option<String>,
    pub content: Option<OfficeContent>,
    pub target_format: OfficeFileFormat,
    pub output_path: String,
    #[serde(default = "empty_json_object")]
    pub options: Value,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficeExportResponse {
    pub output_path: String,
    pub format: OfficeFileFormat,
    #[serde(default)]
    pub diagnostics: Vec<ImportExportDiagnostic>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticSeverity {
    Info,
    Warning,
    Error,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ImportExportDiagnostic {
    pub severity: DiagnosticSeverity,
    pub code: String,
    pub message: String,
    pub location: Option<String>,
    pub recoverable: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficeRecentFile {
    pub artifact: OfficeArtifact,
    pub opened_at: Option<String>,
    pub exists: bool,
    pub thumbnail: Option<OfficeAssetRef>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OfficeBackupMetadata {
    pub id: String,
    pub artifact_id: String,
    pub original_path: String,
    pub backup_path: String,
    pub created_at: Option<String>,
    pub checksum: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OfficeRecoveryMetadata {
    pub id: String,
    pub artifact_id: Option<String>,
    pub product: OfficeProductKind,
    pub original_path: Option<String>,
    pub recovery_path: String,
    pub saved_at: Option<String>,
    pub original_modified_at: Option<String>,
    pub schema_version: String,
    pub checksum: Option<String>,
    pub preview_text: Option<String>,
    pub content_format: OfficeFileFormat,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OfficeRecoverySettings {
    pub enabled: bool,
    pub interval_seconds: u16,
    pub max_files: u16,
    pub retention_days: u16,
    pub directory: Option<String>,
}

impl Default for OfficeRecoverySettings {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_seconds: 120,
            max_files: 10,
            retention_days: 7,
            directory: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficeTemplate {
    pub id: String,
    pub name: String,
    pub product: OfficeProductKind,
    pub format: OfficeFileFormat,
    pub category: String,
    pub description: Option<String>,
    pub builtin: bool,
    pub path: Option<String>,
    pub thumbnail: Option<OfficeAssetRef>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct OfficePromptTemplate {
    pub id: String,
    pub name: String,
    pub product: Option<OfficeProductKind>,
    pub target_task: OfficeAiTask,
    pub category: String,
    pub template: String,
    #[serde(default)]
    pub variables: Vec<PromptTemplateVariable>,
    pub builtin: bool,
    #[serde(default)]
    pub tags: Vec<String>,
    pub version: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PromptTemplateVariable {
    pub key: String,
    pub label: String,
    pub default_value: Option<String>,
    pub required: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OfficeAiTask {
    Writing,
    Rewrite,
    Summarize,
    Translate,
    GrammarCorrection,
    DataAnalysis,
    FormulaGeneration,
    ChartRecommendation,
    SlideGeneration,
    DesignRecommendation,
    ImageGeneration,
    SpeakerScript,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "product", content = "context", rename_all = "snake_case")]
pub enum OfficeAiContext {
    Docs {
        artifact_id: Option<String>,
        selected_text: Option<String>,
        surrounding_text: Option<String>,
        #[serde(default)]
        outline: Vec<String>,
        language: Option<String>,
    },
    Sheets {
        artifact_id: Option<String>,
        sheet_name: Option<String>,
        range: Option<String>,
        #[serde(default)]
        column_headers: Vec<String>,
        #[serde(default)]
        sample_rows: Vec<Vec<String>>,
        current_cell: Option<String>,
    },
    Slides {
        artifact_id: Option<String>,
        slide_id: Option<String>,
        #[serde(default)]
        slide_titles: Vec<String>,
        selected_text: Option<String>,
        speaker_notes: Option<String>,
        content_summary: Option<String>,
    },
    Generic {
        #[serde(default = "empty_json_object")]
        input: Value,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficeAiRequest {
    pub id: String,
    pub task: OfficeAiTask,
    pub prompt: String,
    pub context: OfficeAiContext,
    #[serde(default = "default_true")]
    pub stream: bool,
    #[serde(default)]
    pub allow_external_fallback: bool,
    #[serde(default)]
    pub requires_user_consent: bool,
    pub template_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OfficeAiSuggestion {
    pub id: String,
    pub task: OfficeAiTask,
    pub title: String,
    pub body: String,
    pub patch: Option<Value>,
    pub confidence: Option<f32>,
    #[serde(default)]
    pub diagnostics: Vec<ImportExportDiagnostic>,
    #[serde(default)]
    pub external_provider_used: bool,
}

fn default_true() -> bool {
    true
}

fn empty_json_object() -> Value {
    json!({})
}
