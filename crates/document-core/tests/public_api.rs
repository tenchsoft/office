use tench_document_core::{
    detect_office_file_format, Annotation, AnnotationKind, CellValue, DocumentId, DocumentKind,
    DocumentMetadata, OfficeContent, OfficeFileFormat, OfficeProductKind, RichDocumentContent,
    TextRange,
};
use tench_document_core::{document, office};

#[test]
fn root_reexports_legacy_document_and_office_types() {
    let metadata = DocumentMetadata {
        id: DocumentId("doc_1".to_string()),
        title: "Notes".to_string(),
        kind: DocumentKind::Markdown,
        source_path: None,
        tags: vec!["draft".to_string()],
        created_at: None,
        updated_at: None,
    };
    let annotation = Annotation {
        id: "ann_1".to_string(),
        document_id: metadata.id.clone(),
        kind: AnnotationKind::Comment,
        range: Some(TextRange { start: 0, end: 5 }),
        page: None,
        body: "hello".to_string(),
        tags: vec![],
    };
    let content = OfficeContent::Docs(RichDocumentContent {
        schema: "tench.docs.v1".to_string(),
        document: None,
    });

    assert_eq!(annotation.document_id, metadata.id);
    assert!(matches!(content, OfficeContent::Docs(_)));
    assert_eq!(
        detect_office_file_format("report.docx"),
        Some(OfficeFileFormat::Docx)
    );
    assert!(OfficeProductKind::Docs.supports_format(OfficeFileFormat::Docx));
    assert_eq!(CellValue::Empty, CellValue::Empty);
}

#[test]
fn canonical_modules_export_the_same_public_types() {
    let root_kind = DocumentKind::PlainText;
    let module_kind = document::DocumentKind::PlainText;
    assert_eq!(root_kind, module_kind);

    let root_format = OfficeFileFormat::Xlsx;
    let module_format = office::OfficeFileFormat::Xlsx;
    assert_eq!(root_format, module_format);
    assert_eq!(
        office::primary_office_product_for_format(module_format),
        Some(office::OfficeProductKind::Sheets)
    );
}
