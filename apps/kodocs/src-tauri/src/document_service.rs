use std::fs;
use std::path::PathBuf;

use tench_document_core::{
    detect_office_file_format, OfficeArtifact, OfficeContent, OfficeExportResponse,
    OfficeFileFormat, OfficeOpenResponse, OfficeRecentFile, OfficeRecoveryMetadata,
    OfficeSaveResponse,
};
use tench_office_io::{file_util, office_file, recent_files, recovery};
use tench_office_runtime::{
    export_kodocs_content_bytes, serialize_kodocs_for_target, KODOCS_DISPATCH,
};

use tench_office_io::docs::format as format_io;

const PRODUCT_ID: &str = "tench-kodocs";

pub fn create_document(title: Option<String>) -> OfficeOpenResponse {
    let title = KODOCS_DISPATCH.normalized_title(title);
    let artifact = KODOCS_DISPATCH.new_artifact(title, OfficeFileFormat::Hwpx, None);
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
            KODOCS_DISPATCH.import_content(&raw, fmt)?
        }
        OfficeFileFormat::Hwp
        | OfficeFileFormat::Hwpx
        | OfficeFileFormat::Docx
        | OfficeFileFormat::Odt
        | OfficeFileFormat::Rtf => KODOCS_DISPATCH.import_binary(&path, fmt)?,
        OfficeFileFormat::Pdf => {
            return Err(format!(
                "{} import is planned after the editor and text formats are stable.",
                fmt.extension()
            ));
        }
        _ => {
            return Err(format!(
                "{} is not a Tench Kodocs document format.",
                fmt.extension()
            ));
        }
    };

    let mut artifact = KODOCS_DISPATCH.new_artifact(
        KODOCS_DISPATCH.title_from_path(&path),
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
        serialize_kodocs_for_target,
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
        export_kodocs_content_bytes,
    )
}

pub fn get_recent_documents() -> Result<Vec<OfficeRecentFile>, String> {
    recent_files::get_recent_documents(PRODUCT_ID).map_err(|e| e.to_string())
}

pub fn save_recovery_snapshot(
    artifact: OfficeArtifact,
    content: OfficeContent,
) -> Result<OfficeRecoveryMetadata, String> {
    let preview = KODOCS_DISPATCH.preview_text(&content);
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
    fn create_document_uses_hwpx_format() {
        let opened = create_document(Some("Test HWP".to_string()));
        assert_eq!(opened.artifact.title, "Test HWP");
        assert_eq!(opened.artifact.format, OfficeFileFormat::Hwpx);
    }

    #[test]
    fn save_and_open_docx_round_trips_content() {
        let _guard = isolated_storage("docx_roundtrip");
        let dir = unique_temp_dir("docx_roundtrip_files");
        let target = dir.join("roundtrip.docx");
        let opened = create_document(Some("Roundtrip".to_string()));

        let _saved = save_document(
            opened.artifact,
            opened.content,
            Some(file_util::path_to_string(&target)),
            Some(OfficeFileFormat::Docx),
        )
        .expect("save docx");
        let reopened = open_document(file_util::path_to_string(&target)).expect("open docx");

        assert_eq!(reopened.artifact.title, "roundtrip");
        assert!(matches!(reopened.content, OfficeContent::Docs(_)));
    }

    #[test]
    fn export_to_txt_and_html() {
        let _guard = isolated_storage("exports");
        let dir = unique_temp_dir("export_files");
        let content = docs_content("Export Title", "Export body");

        let txt = dir.join("export.txt");
        let html = dir.join("export.html");

        export_document(
            "kodocs_export".to_string(),
            content.clone(),
            OfficeFileFormat::Txt,
            file_util::path_to_string(&txt),
        )
        .expect("txt export");
        export_document(
            "kodocs_export".to_string(),
            content,
            OfficeFileFormat::Html,
            file_util::path_to_string(&html),
        )
        .expect("html export");

        assert!(fs::read_to_string(txt).unwrap().contains("Export body"));
        assert!(fs::read_to_string(html)
            .unwrap()
            .contains("<h1>Export Title</h1>"));
    }

    fn docs_content(title: &str, body: &str) -> OfficeContent {
        OfficeContent::Docs(RichDocumentContent {
            schema: "tench.kodocs.v1".to_string(),
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

    fn isolated_storage(name: &str) -> std::sync::MutexGuard<'static, ()> {
        let guard = crate::test_util::env_guard();
        let dir = unique_temp_dir(name);
        crate::test_util::set_test_data_home(&dir);
        guard
    }

    fn unique_temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "tench_kodocs_test_{name}_{}_{}",
            std::process::id(),
            file_util::timestamp_millis()
        ));
        fs::create_dir_all(&dir).expect("test temp dir");
        dir
    }
}
