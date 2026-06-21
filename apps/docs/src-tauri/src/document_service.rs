use std::fs;
use std::path::PathBuf;

use tench_document_core::{
    detect_office_file_format, OfficeArtifact, OfficeContent, OfficeExportResponse,
    OfficeFileFormat, OfficeOpenResponse, OfficeRecentFile, OfficeRecoveryMetadata,
    OfficeSaveResponse,
};
use tench_office_io::{file_util, office_file, recent_files, recovery};
use tench_office_runtime::{export_docs_content_bytes, serialize_docs_for_target, DOCS_DISPATCH};

use tench_office_io::docs::format as format_io;

const PRODUCT_ID: &str = "tench-docs";

pub fn create_document(title: Option<String>) -> OfficeOpenResponse {
    let title = DOCS_DISPATCH.normalized_title(title);
    let artifact = DOCS_DISPATCH.new_artifact(title, OfficeFileFormat::Docx, None);
    let content = format_io::empty_docs_content();

    OfficeOpenResponse {
        artifact,
        content,
        diagnostics: Vec::new(),
    }
}

pub fn open_document(path: String) -> Result<OfficeOpenResponse, String> {
    import_document(path, None)
}

pub fn import_document(
    source_path: String,
    source_format: Option<OfficeFileFormat>,
) -> Result<OfficeOpenResponse, String> {
    let path = PathBuf::from(&source_path);
    file_util::ensure_file_exists(&path)?;

    let fmt = source_format
        .or_else(|| detect_office_file_format(&source_path))
        .ok_or_else(|| format!("Unsupported document format: {}", path.display()))?;

    let (content, diagnostics) = match fmt {
        OfficeFileFormat::Txt | OfficeFileFormat::Md | OfficeFileFormat::Html => {
            let raw = fs::read_to_string(&path)
                .map_err(|error| format!("Failed to read {}: {error}", path.display()))?;
            DOCS_DISPATCH.import_content(&raw, fmt)?
        }
        OfficeFileFormat::Docx | OfficeFileFormat::Odt => {
            DOCS_DISPATCH.import_binary(&path, fmt)?
        }
        OfficeFileFormat::Pdf => {
            return Err(format!(
                "{} import is planned after the editor and text formats are stable.",
                fmt.extension()
            ));
        }
        _ => {
            return Err(format!(
                "{} is not a Tench Docs document format.",
                fmt.extension()
            ));
        }
    };

    let mut artifact = DOCS_DISPATCH.new_artifact(
        DOCS_DISPATCH.title_from_path(&path),
        fmt,
        Some(file_util::path_to_string(&path)),
    );
    artifact.updated_at = file_util::modified_at_unix(&path);
    recent_files::add_recent_file(PRODUCT_ID, &artifact)?;

    Ok(OfficeOpenResponse {
        artifact,
        content,
        diagnostics,
    })
}

pub fn save_document(
    artifact: OfficeArtifact,
    content: OfficeContent,
    target_path: Option<String>,
    format: Option<OfficeFileFormat>,
) -> Result<OfficeSaveResponse, String> {
    office_file::save_office_file(
        PRODUCT_ID,
        artifact,
        &content,
        target_path,
        format,
        serialize_docs_for_target,
    )
}

pub fn export_document(
    artifact_id: String,
    content: OfficeContent,
    target_format: OfficeFileFormat,
    output_path: String,
) -> Result<OfficeExportResponse, String> {
    office_file::export_office_file(
        artifact_id,
        &content,
        target_format,
        output_path,
        "exported_from_editor_json",
        export_docs_content_bytes,
    )
}

pub fn get_recent_documents() -> Result<Vec<OfficeRecentFile>, String> {
    recent_files::get_recent_documents(PRODUCT_ID).map_err(|e| e.to_string())
}

pub fn save_recovery_snapshot(
    artifact: OfficeArtifact,
    content: OfficeContent,
) -> Result<OfficeRecoveryMetadata, String> {
    let preview = DOCS_DISPATCH.preview_text(&content);
    office_file::save_recovery_snapshot(PRODUCT_ID, artifact, content, preview)
}

pub fn get_recovery_documents() -> Result<Vec<OfficeRecoveryMetadata>, String> {
    recovery::get_recovery_documents(PRODUCT_ID).map_err(|e| e.to_string())
}

pub fn open_recovery_document(recovery_path: String) -> Result<OfficeOpenResponse, String> {
    office_file::open_recovery_as_unsaved(recovery_path)
}

pub fn delete_recovery_document(recovery_path: String) -> Result<(), String> {
    recovery::delete_recovery_document(PRODUCT_ID, recovery_path).map_err(|e| e.to_string())
}

pub fn clear_recovery_snapshots(artifact_id: String) -> Result<(), String> {
    recovery::clear_recovery_snapshots(PRODUCT_ID, &artifact_id).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use tench_document_core::{
        BlockNode, InlineNode, Marks, ParagraphAttrs, RichDocumentContent, TenchDocument,
    };

    use super::*;

    #[test]
    fn save_and_open_docx_round_trips_content() {
        let _guard = isolated_storage("docx_roundtrip");
        let dir = unique_temp_dir("docx_roundtrip_files");
        let target = dir.join("roundtrip.docx");
        let opened = create_document(Some("Roundtrip".to_string()));

        let saved = save_document(
            opened.artifact,
            opened.content,
            Some(file_util::path_to_string(&target)),
            Some(OfficeFileFormat::Docx),
        )
        .expect("save docx");
        let reopened = open_document(file_util::path_to_string(&target)).expect("open docx");

        assert_eq!(reopened.artifact.title, "roundtrip");
        assert_eq!(
            reopened.artifact.path,
            Some(file_util::path_to_string(&target))
        );
        assert!(!saved.artifact.dirty);
        assert!(matches!(reopened.content, OfficeContent::Docs(_)));
    }

    #[test]
    fn save_existing_document_creates_backup_and_recent_file() {
        let _guard = isolated_storage("backup_recent");
        let dir = unique_temp_dir("backup_recent_files");
        let target = dir.join("backup.docx");
        let opened = create_document(Some("Backup".to_string()));

        let first = save_document(
            opened.artifact,
            opened.content,
            Some(file_util::path_to_string(&target)),
            Some(OfficeFileFormat::Docx),
        )
        .expect("first save");
        let second = save_document(
            first.artifact,
            format_io::plain_text_to_docs_content("Updated"),
            Some(file_util::path_to_string(&target)),
            Some(OfficeFileFormat::Docx),
        )
        .expect("second save");
        let recent = get_recent_documents().expect("recent docs");

        assert!(second.backup.is_some());
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].artifact.title, "Backup");
        assert!(recent[0].exists);
    }

    #[test]
    fn txt_markdown_and_html_exports_write_expected_files() {
        let _guard = isolated_storage("exports");
        let dir = unique_temp_dir("export_files");
        let content = docs_content("Export Title", "Export body");

        let txt = dir.join("export.txt");
        let md = dir.join("export.md");
        let html = dir.join("export.html");

        export_document(
            "doc_export".to_string(),
            content.clone(),
            OfficeFileFormat::Txt,
            file_util::path_to_string(&txt),
        )
        .expect("txt export");
        export_document(
            "doc_export".to_string(),
            content.clone(),
            OfficeFileFormat::Md,
            file_util::path_to_string(&md),
        )
        .expect("md export");
        export_document(
            "doc_export".to_string(),
            content,
            OfficeFileFormat::Html,
            file_util::path_to_string(&html),
        )
        .expect("html export");

        assert!(fs::read_to_string(txt).unwrap().contains("Export body"));
        assert!(fs::read_to_string(md).unwrap().contains("# Export Title"));
        assert!(fs::read_to_string(html)
            .unwrap()
            .contains("<h1>Export Title</h1>"));
    }

    #[test]
    fn docx_export_and_import_preserves_basic_content() {
        let _guard = isolated_storage("docx_export_import");
        let dir = unique_temp_dir("docx_export_import_files");
        let target = dir.join("report.docx");

        let export = export_document(
            "doc_docx".to_string(),
            docs_content("Docx Title", "Docx body"),
            OfficeFileFormat::Docx,
            file_util::path_to_string(&target),
        )
        .expect("docx export");
        let imported =
            import_document(file_util::path_to_string(&target), None).expect("docx import");
        let text = format_io::docs_content_to_plain_text(&imported.content);

        assert_eq!(export.format, OfficeFileFormat::Docx);
        assert!(export
            .diagnostics
            .iter()
            .any(|diag| diag.code == "docx_export"));
        assert_eq!(imported.artifact.format, OfficeFileFormat::Docx);
        assert!(text.contains("Docx Title"));
        assert!(text.contains("Docx body"));
    }

    #[test]
    fn odt_export_and_import_preserves_basic_content() {
        let _guard = isolated_storage("odt_export_import");
        let dir = unique_temp_dir("odt_export_import_files");
        let target = dir.join("report.odt");

        let export = export_document(
            "doc_odt".to_string(),
            docs_content("ODT Title", "ODT body"),
            OfficeFileFormat::Odt,
            file_util::path_to_string(&target),
        )
        .expect("odt export");
        let imported =
            import_document(file_util::path_to_string(&target), None).expect("odt import");
        let text = format_io::docs_content_to_plain_text(&imported.content);

        assert_eq!(export.format, OfficeFileFormat::Odt);
        assert!(export
            .diagnostics
            .iter()
            .any(|diag| diag.code == "odt_export"));
        assert_eq!(imported.artifact.format, OfficeFileFormat::Odt);
        assert!(text.contains("ODT Title"));
        assert!(text.contains("ODT body"));
    }

    #[test]
    fn large_document_exports_to_text_and_docx() {
        let _guard = isolated_storage("large_exports");
        let dir = unique_temp_dir("large_export_files");
        let content = large_docs_content(100);
        let txt = dir.join("large.txt");
        let docx = dir.join("large.docx");
        let odt = dir.join("large.odt");

        export_document(
            "doc_large".to_string(),
            content.clone(),
            OfficeFileFormat::Txt,
            file_util::path_to_string(&txt),
        )
        .expect("large txt export");
        export_document(
            "doc_large".to_string(),
            content,
            OfficeFileFormat::Docx,
            file_util::path_to_string(&docx),
        )
        .expect("large docx export");
        export_document(
            "doc_large".to_string(),
            large_docs_content(100),
            OfficeFileFormat::Odt,
            file_util::path_to_string(&odt),
        )
        .expect("large odt export");

        assert!(fs::metadata(&txt).expect("txt metadata").len() > 10_000);
        assert!(fs::metadata(&docx).expect("docx metadata").len() > 1_000);
        assert!(fs::metadata(&odt).expect("odt metadata").len() > 1_000);
    }

    #[test]
    fn import_text_formats_sets_artifact_metadata() {
        let _guard = isolated_storage("imports");
        let dir = unique_temp_dir("import_files");
        let source = dir.join("notes.md");
        fs::write(&source, "# Notes\n\n- A").expect("write source");

        let response =
            import_document(file_util::path_to_string(&source), None).expect("import markdown");

        assert_eq!(response.artifact.title, "notes");
        assert_eq!(response.artifact.format, OfficeFileFormat::Md);
        assert_eq!(
            response.artifact.path,
            Some(file_util::path_to_string(&source))
        );
        assert!(format_io::docs_content_to_markdown(&response.content).contains("# Notes"));
    }

    #[test]
    fn recovery_snapshots_can_be_listed_opened_and_cleared() {
        let _guard = isolated_storage("recovery");
        let opened = create_document(Some("Recoverable".to_string()));

        let snapshot = save_recovery_snapshot(
            opened.artifact.clone(),
            format_io::plain_text_to_docs_content("Draft body"),
        )
        .expect("save recovery");
        let snapshots = get_recovery_documents().expect("list recovery");
        let recovered =
            open_recovery_document(snapshot.recovery_path.clone()).expect("open recovery");

        assert_eq!(snapshots.len(), 1);
        assert_eq!(snapshots[0].id, snapshot.id);
        assert_eq!(recovered.artifact.title, "Recovered - Recoverable");
        assert_eq!(recovered.artifact.path, None);
        assert!(recovered.artifact.dirty);

        clear_recovery_snapshots(opened.artifact.id).expect("clear recovery");
        assert!(get_recovery_documents()
            .expect("list after clear")
            .is_empty());
    }

    fn docs_content(title: &str, body: &str) -> OfficeContent {
        OfficeContent::Docs(RichDocumentContent {
            schema: "tench.docs.v1".to_string(),
            document: Some(TenchDocument {
                content: vec![
                    BlockNode::Heading {
                        level: 1,
                        content: vec![InlineNode::Text {
                            text: title.to_string(),
                            marks: Marks::default(),
                        }],
                        attrs: ParagraphAttrs::default(),
                    },
                    BlockNode::Paragraph {
                        content: vec![InlineNode::Text {
                            text: body.to_string(),
                            marks: Marks::default(),
                        }],
                        attrs: ParagraphAttrs::default(),
                    },
                ],
                ..TenchDocument::new("")
            }),
        })
    }

    fn large_docs_content(sections: u32) -> OfficeContent {
        let mut blocks = Vec::new();
        for section in 1..=sections {
            blocks.push(BlockNode::Heading {
                level: 2,
                content: vec![InlineNode::Text {
                    text: format!("Section {section}"),
                    marks: Marks::default(),
                }],
                attrs: ParagraphAttrs::default(),
            });
            for paragraph in 1..=5 {
                blocks.push(BlockNode::Paragraph {
                    content: vec![InlineNode::Text {
                        text: format!("Paragraph {paragraph} in section {section} checks serialization, export, recovery, and large document handling for Tench Docs local workflows."),
                        marks: Marks::default(),
                    }],
                    attrs: ParagraphAttrs::default(),
                });
            }
        }

        OfficeContent::Docs(RichDocumentContent {
            schema: "tench.docs.v1".to_string(),
            document: Some(TenchDocument {
                content: blocks,
                ..TenchDocument::new("")
            }),
        })
    }

    fn isolated_storage(name: &str) -> std::sync::MutexGuard<'static, ()> {
        let guard = crate::test_util::env_guard();
        let dir = unique_temp_dir(name);
        crate::test_util::set_test_data_home(&dir);
        guard
    }

    fn unique_temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "tench_docs_test_{name}_{}_{}",
            std::process::id(),
            file_util::timestamp_millis()
        ));
        fs::create_dir_all(&dir).expect("test temp dir");
        dir
    }
}
