#[test]
fn sheets_import_export_api_is_public() {
    let content =
        tench_office_io::sheets::format::csv_to_workbook_content("Name\nAlice", "Contract");

    let csv = tench_office_io::sheets::export_csv_bytes(&content).expect("csv export");
    let html = tench_office_io::sheets::export_html_bytes(&content).expect("html export");
    let pdf = tench_office_io::sheets::export_pdf_bytes(
        &content,
        &tench_office_io::sheets::PdfExportConfig::default(),
    )
    .expect("pdf export");

    assert!(!csv.is_empty());
    assert!(String::from_utf8(html).expect("html").contains("<html>"));
    assert!(pdf.starts_with(b"%PDF"));
}
