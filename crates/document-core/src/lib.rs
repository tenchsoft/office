pub mod document;
pub mod engine;
pub mod office;
pub mod tdm;

pub use document::*;
pub use engine::*;
pub use office::*;
pub use tdm::{
    Alignment, BlockNode, HeadersFooters, ImageSource, InlineNode, ListItem, Margins, Marks,
    Orientation, PageSetup, PaperSize, ParagraphAttrs, StyleDef, TableCell, TableRow, TaskItem,
    TdmMetadata, TenchDocument,
};

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn detects_office_formats_from_paths_and_extensions() {
        assert_eq!(
            detect_office_file_format("/tmp/report.docx"),
            Some(OfficeFileFormat::Docx)
        );
        assert_eq!(
            detect_office_file_format(".xlsx"),
            Some(OfficeFileFormat::Xlsx)
        );
        assert_eq!(
            detect_office_file_format("markdown"),
            Some(OfficeFileFormat::Md)
        );
        assert_eq!(detect_office_file_format("archive.zip"), None);
    }

    #[test]
    fn validates_product_format_support() {
        assert!(OfficeProductKind::Docs.supports_format(OfficeFileFormat::Docx));
        assert!(OfficeProductKind::Sheets.supports_format(OfficeFileFormat::Csv));
        assert!(OfficeProductKind::Slides.supports_format(OfficeFileFormat::Pptx));
        assert!(!OfficeProductKind::Docs.supports_format(OfficeFileFormat::Xlsx));
    }

    #[test]
    fn serializes_office_content_round_trip() {
        let content = OfficeContent::Sheets(WorkbookContent {
            id: "book_1".to_string(),
            title: "Budget".to_string(),
            sheets: vec![SheetContent {
                id: "sheet_1".to_string(),
                name: "Q1".to_string(),
                index: 0,
                cells: vec![CellContent {
                    address: "A1".to_string(),
                    value: CellValue::Number(42.0),
                    formula: Some("=SUM(B1:B4)".to_string()),
                    style: json!({ "bold": true }),
                }],
                row_count: Some(100),
                column_count: Some(10),
            }],
            active_sheet_id: Some("sheet_1".to_string()),
        });

        let encoded = serde_json::to_string(&content).expect("serialize content");
        let decoded: OfficeContent = serde_json::from_str(&encoded).expect("deserialize content");

        assert_eq!(decoded, content);
    }

    #[test]
    fn serializes_ai_contract_without_external_fallback_by_default() {
        let request = OfficeAiRequest {
            id: "ai_1".to_string(),
            task: OfficeAiTask::Summarize,
            prompt: "summarize".to_string(),
            context: OfficeAiContext::Docs {
                artifact_id: Some("doc_1".to_string()),
                selected_text: Some("A long paragraph".to_string()),
                surrounding_text: None,
                outline: vec!["Intro".to_string()],
                language: Some("ko".to_string()),
            },
            stream: true,
            allow_external_fallback: false,
            requires_user_consent: false,
            template_id: None,
        };
        let value = serde_json::to_value(request).expect("ai request json");

        assert_eq!(value["task"], "summarize");
        assert_eq!(value["context"]["product"], "docs");
        assert_eq!(value["allow_external_fallback"], false);
    }

    #[test]
    fn recovery_settings_match_mvp_defaults() {
        let settings = OfficeRecoverySettings::default();

        assert!(settings.enabled);
        assert_eq!(settings.interval_seconds, 120);
        assert_eq!(settings.max_files, 10);
        assert_eq!(settings.retention_days, 7);
    }

    // Edge case tests
    #[test]
    fn empty_string_extension_returns_none() {
        assert_eq!(OfficeFileFormat::from_extension(""), None);
        assert_eq!(detect_office_file_format(""), None);
    }

    #[test]
    fn whitespace_only_extension_returns_none() {
        assert_eq!(OfficeFileFormat::from_extension("   "), None);
    }

    #[test]
    fn unknown_extension_returns_none() {
        assert_eq!(OfficeFileFormat::from_extension("xyz"), None);
        assert_eq!(detect_office_file_format("file.unknown"), None);
    }

    #[test]
    fn extension_with_leading_dot_and_whitespace() {
        assert_eq!(
            OfficeFileFormat::from_extension("  .DOCX  "),
            Some(OfficeFileFormat::Docx)
        );
    }

    #[test]
    fn all_office_file_format_variants_roundtrip() {
        for variant in [
            OfficeFileFormat::Docx,
            OfficeFileFormat::Txt,
            OfficeFileFormat::Md,
            OfficeFileFormat::Html,
            OfficeFileFormat::Odt,
            OfficeFileFormat::Xlsx,
            OfficeFileFormat::Csv,
            OfficeFileFormat::Tsv,
            OfficeFileFormat::Ods,
            OfficeFileFormat::Pptx,
            OfficeFileFormat::Odp,
            OfficeFileFormat::Pdf,
            OfficeFileFormat::Rtf,
            OfficeFileFormat::Epub,
            OfficeFileFormat::Images,
            OfficeFileFormat::Video,
        ] {
            let serialized = serde_json::to_string(&variant).unwrap();
            let deserialized: OfficeFileFormat = serde_json::from_str(&serialized).unwrap();
            assert_eq!(variant, deserialized);
        }
    }

    #[test]
    fn all_document_kind_variants_roundtrip() {
        for variant in [
            DocumentKind::PlainText,
            DocumentKind::Markdown,
            DocumentKind::RichText,
            DocumentKind::Pdf,
            DocumentKind::Docx,
            DocumentKind::Code,
            DocumentKind::WebPage,
            DocumentKind::Image,
            DocumentKind::Video,
        ] {
            let serialized = serde_json::to_string(&variant).unwrap();
            let deserialized: DocumentKind = serde_json::from_str(&serialized).unwrap();
            assert_eq!(variant, deserialized);
        }
    }

    #[test]
    fn all_office_product_kind_variants_roundtrip() {
        for variant in [
            OfficeProductKind::Docs,
            OfficeProductKind::Sheets,
            OfficeProductKind::Slides,
        ] {
            let serialized = serde_json::to_string(&variant).unwrap();
            let deserialized: OfficeProductKind = serde_json::from_str(&serialized).unwrap();
            assert_eq!(variant, deserialized);
            assert!(!variant.product_id().is_empty());
        }
    }

    #[test]
    fn all_annotation_kind_variants_roundtrip() {
        for variant in [
            AnnotationKind::Highlight,
            AnnotationKind::Comment,
            AnnotationKind::Drawing,
            AnnotationKind::Bookmark,
            AnnotationKind::Task,
        ] {
            let serialized = serde_json::to_string(&variant).unwrap();
            let deserialized: AnnotationKind = serde_json::from_str(&serialized).unwrap();
            assert_eq!(variant, deserialized);
        }
    }

    #[test]
    fn all_diagnostic_severity_variants_roundtrip() {
        for variant in [
            DiagnosticSeverity::Info,
            DiagnosticSeverity::Warning,
            DiagnosticSeverity::Error,
        ] {
            let serialized = serde_json::to_string(&variant).unwrap();
            let deserialized: DiagnosticSeverity = serde_json::from_str(&serialized).unwrap();
            assert_eq!(variant, deserialized);
        }
    }

    #[test]
    fn all_office_asset_kind_variants_roundtrip() {
        for variant in [
            OfficeAssetKind::Image,
            OfficeAssetKind::Video,
            OfficeAssetKind::Audio,
            OfficeAssetKind::Font,
            OfficeAssetKind::Thumbnail,
            OfficeAssetKind::GeneratedImage,
        ] {
            let serialized = serde_json::to_string(&variant).unwrap();
            let deserialized: OfficeAssetKind = serde_json::from_str(&serialized).unwrap();
            assert_eq!(variant, deserialized);
        }
    }

    #[test]
    fn all_office_ai_task_variants_roundtrip() {
        for variant in [
            OfficeAiTask::Writing,
            OfficeAiTask::Rewrite,
            OfficeAiTask::Summarize,
            OfficeAiTask::Translate,
            OfficeAiTask::GrammarCorrection,
            OfficeAiTask::DataAnalysis,
            OfficeAiTask::FormulaGeneration,
            OfficeAiTask::ChartRecommendation,
            OfficeAiTask::SlideGeneration,
            OfficeAiTask::DesignRecommendation,
            OfficeAiTask::ImageGeneration,
            OfficeAiTask::SpeakerScript,
        ] {
            let serialized = serde_json::to_string(&variant).unwrap();
            let deserialized: OfficeAiTask = serde_json::from_str(&serialized).unwrap();
            assert_eq!(variant, deserialized);
        }
    }

    #[test]
    fn empty_collections_and_none_fields() {
        let artifact = OfficeArtifact {
            id: String::new(),
            title: String::new(),
            product: OfficeProductKind::Docs,
            format: OfficeFileFormat::Docx,
            path: None,
            schema_version: String::new(),
            created_at: None,
            updated_at: None,
            dirty: false,
            tags: vec![],
            assets: vec![],
        };
        let serialized = serde_json::to_string(&artifact).unwrap();
        let deserialized: OfficeArtifact = serde_json::from_str(&serialized).unwrap();
        assert_eq!(artifact, deserialized);
    }

    #[test]
    fn text_range_zero_values() {
        let range = TextRange { start: 0, end: 0 };
        let serialized = serde_json::to_string(&range).unwrap();
        let deserialized: TextRange = serde_json::from_str(&serialized).unwrap();
        assert_eq!(range, deserialized);
    }

    #[test]
    fn cell_value_empty_roundtrip() {
        let value = CellValue::Empty;
        let serialized = serde_json::to_string(&value).unwrap();
        let deserialized: CellValue = serde_json::from_str(&serialized).unwrap();
        assert_eq!(value, deserialized);
    }

    #[test]
    fn cell_value_negative_number() {
        let value = CellValue::Number(-12345.6789);
        let serialized = serde_json::to_string(&value).unwrap();
        let deserialized: CellValue = serde_json::from_str(&serialized).unwrap();
        assert_eq!(value, deserialized);
    }

    #[test]
    fn cell_value_very_large_number() {
        let value = CellValue::Number(1e308);
        let serialized = serde_json::to_string(&value).unwrap();
        let deserialized: CellValue = serde_json::from_str(&serialized).unwrap();
        assert_eq!(value, deserialized);
    }

    #[test]
    fn office_content_docs_empty_roundtrip() {
        let content = OfficeContent::Docs(RichDocumentContent {
            schema: String::new(),
            document: None,
        });
        let serialized = serde_json::to_string(&content).unwrap();
        let deserialized: OfficeContent = serde_json::from_str(&serialized).unwrap();
        assert_eq!(content, deserialized);
    }

    #[test]
    fn office_content_slides_empty_roundtrip() {
        let content = OfficeContent::Slides(PresentationContent {
            id: String::new(),
            title: String::new(),
            width: 0,
            height: 0,
            slides: vec![],
            assets: vec![],
        });
        let serialized = serde_json::to_string(&content).unwrap();
        let deserialized: OfficeContent = serde_json::from_str(&serialized).unwrap();
        assert_eq!(content, deserialized);
    }

    #[test]
    fn office_save_options_defaults() {
        let options = OfficeSaveOptions {
            target_path: None,
            format: None,
            atomic: true,
            create_backup: true,
            update_recent: true,
        };
        let serialized = serde_json::to_string(&options).unwrap();
        let deserialized: OfficeSaveOptions = serde_json::from_str(&serialized).unwrap();
        assert_eq!(options, deserialized);
    }

    #[test]
    fn office_ai_context_generic_empty_roundtrip() {
        let context = OfficeAiContext::Generic { input: json!({}) };
        let serialized = serde_json::to_string(&context).unwrap();
        let deserialized: OfficeAiContext = serde_json::from_str(&serialized).unwrap();
        assert_eq!(context, deserialized);
    }

    #[test]
    fn office_ai_context_docs_empty_collections() {
        let context = OfficeAiContext::Docs {
            artifact_id: None,
            selected_text: None,
            surrounding_text: None,
            outline: vec![],
            language: None,
        };
        let serialized = serde_json::to_string(&context).unwrap();
        let deserialized: OfficeAiContext = serde_json::from_str(&serialized).unwrap();
        assert_eq!(context, deserialized);
    }

    #[test]
    fn office_ai_context_sheets_empty_collections() {
        let context = OfficeAiContext::Sheets {
            artifact_id: None,
            sheet_name: None,
            range: None,
            column_headers: vec![],
            sample_rows: vec![],
            current_cell: None,
        };
        let serialized = serde_json::to_string(&context).unwrap();
        let deserialized: OfficeAiContext = serde_json::from_str(&serialized).unwrap();
        assert_eq!(context, deserialized);
    }

    #[test]
    fn office_ai_context_slides_empty_collections() {
        let context = OfficeAiContext::Slides {
            artifact_id: None,
            slide_id: None,
            slide_titles: vec![],
            selected_text: None,
            speaker_notes: None,
            content_summary: None,
        };
        let serialized = serde_json::to_string(&context).unwrap();
        let deserialized: OfficeAiContext = serde_json::from_str(&serialized).unwrap();
        assert_eq!(context, deserialized);
    }

    #[test]
    fn primary_office_product_for_format_pdf_returns_none() {
        assert_eq!(
            primary_office_product_for_format(OfficeFileFormat::Pdf),
            None
        );
    }

    #[test]
    fn workbook_content_empty_sheets_roundtrip() {
        let content = WorkbookContent {
            id: String::new(),
            title: String::new(),
            sheets: vec![],
            active_sheet_id: None,
        };
        let serialized = serde_json::to_string(&content).unwrap();
        let deserialized: WorkbookContent = serde_json::from_str(&serialized).unwrap();
        assert_eq!(content, deserialized);
    }

    #[test]
    fn sheet_content_zero_index_and_none_counts() {
        let sheet = SheetContent {
            id: String::new(),
            name: String::new(),
            index: 0,
            cells: vec![],
            row_count: None,
            column_count: None,
        };
        let serialized = serde_json::to_string(&sheet).unwrap();
        let deserialized: SheetContent = serde_json::from_str(&serialized).unwrap();
        assert_eq!(sheet, deserialized);
    }

    #[test]
    fn office_rect_zero_values() {
        let rect = OfficeRect {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        };
        let serialized = serde_json::to_string(&rect).unwrap();
        let deserialized: OfficeRect = serde_json::from_str(&serialized).unwrap();
        assert_eq!(rect, deserialized);
    }

    #[test]
    fn office_rect_negative_and_large_values() {
        let rect = OfficeRect {
            x: -9999.99,
            y: -0.0001,
            width: 1e30,
            height: f32::MAX,
        };
        let serialized = serde_json::to_string(&rect).unwrap();
        let deserialized: OfficeRect = serde_json::from_str(&serialized).unwrap();
        assert_eq!(rect, deserialized);
    }

    #[test]
    fn slide_content_empty_objects() {
        let slide = SlideContent {
            id: String::new(),
            index: 0,
            title: None,
            notes: None,
            objects: vec![],
            background: json!({}),
            transition: json!({}),
        };
        let serialized = serde_json::to_string(&slide).unwrap();
        let deserialized: SlideContent = serde_json::from_str(&serialized).unwrap();
        assert_eq!(slide, deserialized);
    }

    #[test]
    fn slide_object_empty_data() {
        let obj = SlideObject {
            id: String::new(),
            object_type: String::new(),
            bounds: OfficeRect {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
            },
            data: json!({}),
            style: json!({}),
        };
        let serialized = serde_json::to_string(&obj).unwrap();
        let deserialized: SlideObject = serde_json::from_str(&serialized).unwrap();
        assert_eq!(obj, deserialized);
    }

    #[test]
    fn office_asset_ref_none_fields() {
        let asset = OfficeAssetRef {
            id: String::new(),
            kind: OfficeAssetKind::Image,
            path: None,
            mime_type: None,
            checksum: None,
            metadata: json!({}),
        };
        let serialized = serde_json::to_string(&asset).unwrap();
        let deserialized: OfficeAssetRef = serde_json::from_str(&serialized).unwrap();
        assert_eq!(asset, deserialized);
    }

    #[test]
    fn office_create_request_empty_title() {
        let req = OfficeCreateRequest {
            product: OfficeProductKind::Docs,
            title: String::new(),
            template_id: None,
        };
        let serialized = serde_json::to_string(&req).unwrap();
        let deserialized: OfficeCreateRequest = serde_json::from_str(&serialized).unwrap();
        assert_eq!(req, deserialized);
    }

    #[test]
    fn office_open_request_empty_path() {
        let req = OfficeOpenRequest {
            path: String::new(),
            product_hint: None,
        };
        let serialized = serde_json::to_string(&req).unwrap();
        let deserialized: OfficeOpenRequest = serde_json::from_str(&serialized).unwrap();
        assert_eq!(req, deserialized);
    }

    #[test]
    fn office_import_request_empty_source_path() {
        let req = OfficeImportRequest {
            source_path: String::new(),
            target_product: OfficeProductKind::Sheets,
            source_format: None,
        };
        let serialized = serde_json::to_string(&req).unwrap();
        let deserialized: OfficeImportRequest = serde_json::from_str(&serialized).unwrap();
        assert_eq!(req, deserialized);
    }

    #[test]
    fn office_export_request_empty_paths() {
        let req = OfficeExportRequest {
            artifact_id: String::new(),
            source_path: None,
            content: None,
            target_format: OfficeFileFormat::Pdf,
            output_path: String::new(),
            options: json!({}),
        };
        let serialized = serde_json::to_string(&req).unwrap();
        let deserialized: OfficeExportRequest = serde_json::from_str(&serialized).unwrap();
        assert_eq!(req, deserialized);
    }

    #[test]
    fn import_export_diagnostic_empty_location() {
        let diag = ImportExportDiagnostic {
            severity: DiagnosticSeverity::Info,
            code: String::new(),
            message: String::new(),
            location: None,
            recoverable: false,
        };
        let serialized = serde_json::to_string(&diag).unwrap();
        let deserialized: ImportExportDiagnostic = serde_json::from_str(&serialized).unwrap();
        assert_eq!(diag, deserialized);
    }

    #[test]
    fn office_backup_metadata_none_fields() {
        let meta = OfficeBackupMetadata {
            id: String::new(),
            artifact_id: String::new(),
            original_path: String::new(),
            backup_path: String::new(),
            created_at: None,
            checksum: None,
        };
        let serialized = serde_json::to_string(&meta).unwrap();
        let deserialized: OfficeBackupMetadata = serde_json::from_str(&serialized).unwrap();
        assert_eq!(meta, deserialized);
    }

    #[test]
    fn office_recovery_metadata_none_artifact_id() {
        let meta = OfficeRecoveryMetadata {
            id: String::new(),
            artifact_id: None,
            product: OfficeProductKind::Docs,
            original_path: None,
            recovery_path: String::new(),
            saved_at: None,
            original_modified_at: None,
            schema_version: String::new(),
            checksum: None,
            preview_text: None,
            content_format: OfficeFileFormat::Docx,
        };
        let serialized = serde_json::to_string(&meta).unwrap();
        let deserialized: OfficeRecoveryMetadata = serde_json::from_str(&serialized).unwrap();
        assert_eq!(meta, deserialized);
    }

    #[test]
    fn office_template_empty_description_and_path() {
        let template = OfficeTemplate {
            id: String::new(),
            name: String::new(),
            product: OfficeProductKind::Slides,
            format: OfficeFileFormat::Pptx,
            category: String::new(),
            description: None,
            builtin: false,
            path: None,
            thumbnail: None,
            tags: vec![],
        };
        let serialized = serde_json::to_string(&template).unwrap();
        let deserialized: OfficeTemplate = serde_json::from_str(&serialized).unwrap();
        assert_eq!(template, deserialized);
    }

    #[test]
    fn office_prompt_template_empty_variables() {
        let template = OfficePromptTemplate {
            id: String::new(),
            name: String::new(),
            product: None,
            target_task: OfficeAiTask::Writing,
            category: String::new(),
            template: String::new(),
            variables: vec![],
            builtin: false,
            tags: vec![],
            version: String::new(),
        };
        let serialized = serde_json::to_string(&template).unwrap();
        let deserialized: OfficePromptTemplate = serde_json::from_str(&serialized).unwrap();
        assert_eq!(template, deserialized);
    }

    #[test]
    fn prompt_template_variable_none_default() {
        let var = PromptTemplateVariable {
            key: String::new(),
            label: String::new(),
            default_value: None,
            required: false,
        };
        let serialized = serde_json::to_string(&var).unwrap();
        let deserialized: PromptTemplateVariable = serde_json::from_str(&serialized).unwrap();
        assert_eq!(var, deserialized);
    }

    #[test]
    fn document_id_empty_roundtrip() {
        let id = DocumentId(String::new());
        let serialized = serde_json::to_string(&id).unwrap();
        let deserialized: DocumentId = serde_json::from_str(&serialized).unwrap();
        assert_eq!(id, deserialized);
    }

    #[test]
    fn document_metadata_empty_tags_and_none_dates() {
        let meta = DocumentMetadata {
            id: DocumentId(String::new()),
            title: String::new(),
            kind: DocumentKind::PlainText,
            source_path: None,
            tags: vec![],
            created_at: None,
            updated_at: None,
        };
        let serialized = serde_json::to_string(&meta).unwrap();
        let deserialized: DocumentMetadata = serde_json::from_str(&serialized).unwrap();
        assert_eq!(meta, deserialized);
    }

    #[test]
    fn annotation_none_range_and_page() {
        let ann = Annotation {
            id: String::new(),
            document_id: DocumentId(String::new()),
            kind: AnnotationKind::Comment,
            range: None,
            page: None,
            body: String::new(),
            tags: vec![],
        };
        let serialized = serde_json::to_string(&ann).unwrap();
        let deserialized: Annotation = serde_json::from_str(&serialized).unwrap();
        assert_eq!(ann, deserialized);
    }

    #[test]
    fn office_recent_file_none_thumbnail() {
        let recent = OfficeRecentFile {
            artifact: OfficeArtifact {
                id: String::new(),
                title: String::new(),
                product: OfficeProductKind::Docs,
                format: OfficeFileFormat::Docx,
                path: None,
                schema_version: String::new(),
                created_at: None,
                updated_at: None,
                dirty: false,
                tags: vec![],
                assets: vec![],
            },
            opened_at: None,
            exists: false,
            thumbnail: None,
        };
        let serialized = serde_json::to_string(&recent).unwrap();
        let deserialized: OfficeRecentFile = serde_json::from_str(&serialized).unwrap();
        assert_eq!(recent, deserialized);
    }

    #[test]
    fn office_save_response_none_backup() {
        let resp = OfficeSaveResponse {
            artifact: OfficeArtifact {
                id: String::new(),
                title: String::new(),
                product: OfficeProductKind::Docs,
                format: OfficeFileFormat::Docx,
                path: None,
                schema_version: String::new(),
                created_at: None,
                updated_at: None,
                dirty: false,
                tags: vec![],
                assets: vec![],
            },
            backup: None,
            diagnostics: vec![],
        };
        let serialized = serde_json::to_string(&resp).unwrap();
        let deserialized: OfficeSaveResponse = serde_json::from_str(&serialized).unwrap();
        assert_eq!(resp, deserialized);
    }

    #[test]
    fn office_open_response_empty_diagnostics() {
        let resp = OfficeOpenResponse {
            artifact: OfficeArtifact {
                id: String::new(),
                title: String::new(),
                product: OfficeProductKind::Docs,
                format: OfficeFileFormat::Docx,
                path: None,
                schema_version: String::new(),
                created_at: None,
                updated_at: None,
                dirty: false,
                tags: vec![],
                assets: vec![],
            },
            content: OfficeContent::Docs(RichDocumentContent {
                schema: String::new(),
                document: None,
            }),
            diagnostics: vec![],
        };
        let serialized = serde_json::to_string(&resp).unwrap();
        let deserialized: OfficeOpenResponse = serde_json::from_str(&serialized).unwrap();
        assert_eq!(resp, deserialized);
    }

    #[test]
    fn office_import_response_empty_diagnostics() {
        let resp = OfficeImportResponse {
            artifact: OfficeArtifact {
                id: String::new(),
                title: String::new(),
                product: OfficeProductKind::Docs,
                format: OfficeFileFormat::Docx,
                path: None,
                schema_version: String::new(),
                created_at: None,
                updated_at: None,
                dirty: false,
                tags: vec![],
                assets: vec![],
            },
            content: OfficeContent::Docs(RichDocumentContent {
                schema: String::new(),
                document: None,
            }),
            diagnostics: vec![],
        };
        let serialized = serde_json::to_string(&resp).unwrap();
        let deserialized: OfficeImportResponse = serde_json::from_str(&serialized).unwrap();
        assert_eq!(resp, deserialized);
    }

    #[test]
    fn office_export_response_empty_diagnostics() {
        let resp = OfficeExportResponse {
            output_path: String::new(),
            format: OfficeFileFormat::Pdf,
            diagnostics: vec![],
        };
        let serialized = serde_json::to_string(&resp).unwrap();
        let deserialized: OfficeExportResponse = serde_json::from_str(&serialized).unwrap();
        assert_eq!(resp, deserialized);
    }

    #[test]
    fn office_ai_request_empty_prompt() {
        let req = OfficeAiRequest {
            id: String::new(),
            task: OfficeAiTask::Writing,
            prompt: String::new(),
            context: OfficeAiContext::Generic { input: json!({}) },
            stream: false,
            allow_external_fallback: false,
            requires_user_consent: false,
            template_id: None,
        };
        let serialized = serde_json::to_string(&req).unwrap();
        let deserialized: OfficeAiRequest = serde_json::from_str(&serialized).unwrap();
        assert_eq!(req, deserialized);
    }

    #[test]
    fn office_ai_suggestion_none_patch_and_confidence() {
        let sug = OfficeAiSuggestion {
            id: String::new(),
            task: OfficeAiTask::Summarize,
            title: String::new(),
            body: String::new(),
            patch: None,
            confidence: None,
            diagnostics: vec![],
            external_provider_used: false,
        };
        let serialized = serde_json::to_string(&sug).unwrap();
        let deserialized: OfficeAiSuggestion = serde_json::from_str(&serialized).unwrap();
        assert_eq!(sug, deserialized);
    }
}
