use crate::error::HwpError;
use std::io::Write;
use tench_document_core::tdm::*;

pub fn write_hwpx(doc: &TenchDocument) -> Result<Vec<u8>, HwpError> {
    let header_xml = build_header_xml(doc);
    let section_xml = build_section_xml(doc);

    let mut buf = Vec::new();
    {
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut buf));

        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);

        // mimetype (must be first, uncompressed)
        zip.start_file("mimetype", zip::write::SimpleFileOptions::default())
            .map_err(HwpError::Zip)?;
        zip.write_all(b"application/hwp+zip")
            .map_err(HwpError::Io)?;

        // version.xml
        zip.start_file("version.xml", options)
            .map_err(HwpError::Zip)?;
        zip.write_all(
            b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<HWPVersion Major=\"1\" Minor=\"5\"/>",
        )
        .map_err(HwpError::Io)?;

        // Contents/content.hpf
        let hpf = build_content_hpf(doc);
        zip.start_file("Contents/content.hpf", options)
            .map_err(HwpError::Zip)?;
        zip.write_all(hpf.as_bytes()).map_err(HwpError::Io)?;

        // Contents/header.xml
        zip.start_file("Contents/header.xml", options)
            .map_err(HwpError::Zip)?;
        zip.write_all(header_xml.as_bytes()).map_err(HwpError::Io)?;

        // Contents/section0.xml
        zip.start_file("Contents/section0.xml", options)
            .map_err(HwpError::Zip)?;
        zip.write_all(section_xml.as_bytes())
            .map_err(HwpError::Io)?;

        zip.finish().map_err(HwpError::Zip)?;
    }

    Ok(buf)
}

fn build_content_hpf(_doc: &TenchDocument) -> String {
    r#"<?xml version="1.0" encoding="UTF-8"?>
<opf:package xmlns:opf="http://www.idpf.org/2007/opf/" version="" unique-identifier="">
  <opf:metadata>
    <opf:language>ko</opf:language>
  </opf:metadata>
  <opf:manifest>
    <opf:item id="header" href="Contents/header.xml" media-type="application/xml"/>
    <opf:item id="section0" href="Contents/section0.xml" media-type="application/xml"/>
  </opf:manifest>
  <opf:spine>
    <opf:itemref idref="header" linear="yes"/>
    <opf:itemref idref="section0" linear="yes"/>
  </opf:spine>
</opf:package>"#
        .to_string()
}

fn build_header_xml(doc: &TenchDocument) -> String {
    let title = xml_escape(&doc.metadata.title);
    let author = doc.metadata.author.as_deref().unwrap_or("");
    let author = xml_escape(author);
    let created = doc.metadata.created_at.as_deref().unwrap_or("");
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<hh:head version="1.5" secCnt="1"
  xmlns:ha="http://www.hancom.co.kr/hwpml/2011/app"
  xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
  xmlns:hh="http://www.hancom.co.kr/hwpml/2011/head">
  <hh:beginNum page="1" footnote="1" endnote="1" pic="1" tbl="1" equation="1"/>
  <hh:refList>
    <hh:fontfaces itemCnt="1">
      <hh:fontface lang="hangul" fontCnt="1">
        <hh:font id="0" face="맑은 고딕" type="ttf" isEmbedded="0"/>
      </hh:fontface>
    </hh:fontfaces>
    <hh:charProperties itemCnt="1">
      <hh:charPr id="0" height="1000" textColor="0">
        <hh:fontRef hangul="0" latin="0" hanja="0"/>
      </hh:charPr>
    </hh:charProperties>
    <hh:paraProperties itemCnt="1">
      <hh:paraPr id="0">
        <hh:align horizontal="LEFT"/>
        <hh:lineSpacing type="percent" value="160"/>
      </hh:paraPr>
    </hh:paraProperties>
    <hh:styles itemCnt="1">
      <hh:style id="0" type="para" name="바탕글" engName="Normal"/>
    </hh:styles>
  </hh:refList>
  <hh:docSetting>
    <hh:title>{title}</hh:title>
    <hh:author>{author}</hh:author>
    <hh:date>{created}</hh:date>
  </hh:docSetting>
</hh:head>"#
    )
}

fn build_section_xml(doc: &TenchDocument) -> String {
    let mut xml = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"
        xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
"#,
    );

    for block in &doc.content {
        match block {
            BlockNode::Paragraph { content, attrs } => {
                let align = alignment_to_hwp(attrs.alignment);
                xml.push_str(&format!(
                    "  <hp:p paraPrIDRef=\"0\" styleIDRef=\"0\" align=\"{}\">\n",
                    align
                ));
                xml.push_str("    <hp:run charPrIDRef=\"0\">\n");
                for inline in content {
                    write_inline_node(&mut xml, inline);
                }
                xml.push_str("    </hp:run>\n");
                xml.push_str("  </hp:p>\n");
            }
            BlockNode::Heading {
                level,
                content,
                attrs,
            } => {
                let align = alignment_to_hwp(attrs.alignment);
                xml.push_str(&format!(
                    "  <hp:p paraPrIDRef=\"0\" styleIDRef=\"0\" align=\"{}\" heading=\"{}\">\n",
                    align, level
                ));
                xml.push_str("    <hp:run charPrIDRef=\"0\">\n");
                for inline in content {
                    write_inline_node(&mut xml, inline);
                }
                xml.push_str("    </hp:run>\n");
                xml.push_str("  </hp:p>\n");
            }
            BlockNode::Table { rows } => {
                xml.push_str("  <hp:p>\n    <hp:run charPrIDRef=\"0\">\n");
                xml.push_str(&format!(
                    "      <hp:tbl rowCnt=\"{}\" colCnt=\"{}\" cellSpacing=\"0\">\n",
                    rows.len(),
                    rows.first().map(|r| r.cells.len()).unwrap_or(0)
                ));
                for row in rows {
                    xml.push_str("        <hp:tr>\n");
                    for cell in &row.cells {
                        xml.push_str("          <hp:tc name=\"\" header=\"0\" hasMargin=\"0\">\n");
                        xml.push_str("            <hp:subList>\n");
                        for content_block in &cell.content {
                            if let BlockNode::Paragraph { content, .. } = content_block {
                                xml.push_str(
                                    "              <hp:p paraPrIDRef=\"0\" styleIDRef=\"0\">\n",
                                );
                                xml.push_str("                <hp:run charPrIDRef=\"0\">\n");
                                for inline in content {
                                    write_inline_node(&mut xml, inline);
                                }
                                xml.push_str("                </hp:run>\n");
                                xml.push_str("              </hp:p>\n");
                            }
                        }
                        xml.push_str("            </hp:subList>\n");
                        xml.push_str("          </hp:tc>\n");
                    }
                    xml.push_str("        </hp:tr>\n");
                }
                xml.push_str("      </hp:tbl>\n");
                xml.push_str("    </hp:run>\n  </hp:p>\n");
            }
            BlockNode::Image {
                source,
                alt,
                width,
                height,
            } => {
                let w = width.unwrap_or(100.0);
                let h = height.unwrap_or(100.0);
                let alt_text = alt.as_deref().unwrap_or("");
                xml.push_str("  <hp:p>\n    <hp:run charPrIDRef=\"0\">\n");
                // Write image reference
                match source {
                    ImageSource::Embedded { data } => {
                        // For embedded images, write base64 inline
                        let b64 = base64_encode(data);
                        xml.push_str(&format!(
                            "      <hp:img width=\"{}\" height=\"{}\" alt=\"{}\" data=\"{}\"/>\n",
                            w as i32,
                            h as i32,
                            xml_escape(alt_text),
                            b64
                        ));
                    }
                    ImageSource::Referenced { path } => {
                        xml.push_str(&format!(
                            "      <hp:img width=\"{}\" height=\"{}\" alt=\"{}\" ref=\"{}\"/>\n",
                            w as i32,
                            h as i32,
                            xml_escape(alt_text),
                            xml_escape(path)
                        ));
                    }
                }
                xml.push_str("    </hp:run>\n  </hp:p>\n");
            }
            BlockNode::CodeBlock { language, code } => {
                let lang_attr = language
                    .as_deref()
                    .map(|l| format!(" language=\"{}\"", xml_escape(l)))
                    .unwrap_or_default();
                xml.push_str("  <hp:p paraPrIDRef=\"0\" styleIDRef=\"0\">\n");
                xml.push_str(&format!(
                    "    <hp:run charPrIDRef=\"0\"><hp:t code=\"1\"{}>{}</hp:t></hp:run>\n",
                    lang_attr,
                    xml_escape(code)
                ));
                xml.push_str("  </hp:p>\n");
            }
            BlockNode::BulletList { items } | BlockNode::OrderedList { items, .. } => {
                for item in items {
                    let text: String = item
                        .content
                        .iter()
                        .filter_map(|n| match n {
                            InlineNode::Text { text, .. } => Some(text.as_str()),
                            _ => None,
                        })
                        .collect();
                    if !text.is_empty() {
                        xml.push_str("  <hp:p paraPrIDRef=\"0\" styleIDRef=\"0\">\n");
                        xml.push_str("    <hp:run charPrIDRef=\"0\"><hp:t>• </hp:t></hp:run>\n");
                        xml.push_str(&format!(
                            "    <hp:run charPrIDRef=\"0\"><hp:t>{}</hp:t></hp:run>\n",
                            xml_escape(&text)
                        ));
                        xml.push_str("  </hp:p>\n");
                    }
                }
            }
            BlockNode::HorizontalRule => {
                xml.push_str("  <hp:p paraPrIDRef=\"0\" styleIDRef=\"0\">\n");
                xml.push_str(
                    "    <hp:run charPrIDRef=\"0\"><hp:t>————————————————</hp:t></hp:run>\n",
                );
                xml.push_str("  </hp:p>\n");
            }
            BlockNode::PageBreak => {
                xml.push_str("  <hp:p paraPrIDRef=\"0\" styleIDRef=\"0\">\n");
                xml.push_str("    <hp:run charPrIDRef=\"0\"><hp:columnBreak/></hp:run>\n");
                xml.push_str("  </hp:p>\n");
            }
            BlockNode::Footnote { number, content } => {
                let text: String = content
                    .iter()
                    .filter_map(|n| match n {
                        InlineNode::Text { text, .. } => Some(text.as_str()),
                        _ => None,
                    })
                    .collect();
                xml.push_str("  <hp:p paraPrIDRef=\"0\" styleIDRef=\"0\">\n");
                xml.push_str(&format!(
                    "    <hp:run charPrIDRef=\"0\"><hp:footnote number=\"{}\">{}</hp:footnote></hp:run>\n",
                    number, xml_escape(&text)
                ));
                xml.push_str("  </hp:p>\n");
            }
            BlockNode::BlockQuote { content } => {
                for child in content {
                    if let Some(text) = extract_block_text_for_quote(child) {
                        xml.push_str("  <hp:p paraPrIDRef=\"0\" styleIDRef=\"0\">\n");
                        xml.push_str(&format!(
                            "    <hp:run charPrIDRef=\"0\"><hp:t>{}</hp:t></hp:run>\n",
                            xml_escape(&text)
                        ));
                        xml.push_str("  </hp:p>\n");
                    }
                }
            }
            BlockNode::TaskList { items } => {
                for item in items {
                    let check = if item.checked { "☑" } else { "☐" };
                    let text: String = item
                        .content
                        .iter()
                        .filter_map(|n| match n {
                            InlineNode::Text { text, .. } => Some(text.as_str()),
                            _ => None,
                        })
                        .collect();
                    xml.push_str("  <hp:p paraPrIDRef=\"0\" styleIDRef=\"0\">\n");
                    xml.push_str(&format!(
                        "    <hp:run charPrIDRef=\"0\"><hp:t>{} {}</hp:t></hp:run>\n",
                        check,
                        xml_escape(&text)
                    ));
                    xml.push_str("  </hp:p>\n");
                }
            }
        }
    }

    // Section properties - use actual document page setup
    let ps = &doc.page_setup;
    let (w_mm, h_mm) = ps.paper_size.dimensions_mm();
    let (w, h) = match ps.orientation {
        tench_document_core::Orientation::Portrait => (w_mm, h_mm),
        tench_document_core::Orientation::Landscape => (h_mm, w_mm),
    };
    let mm_to_hwpu = |mm: f32| -> i32 { (mm as f64 * 7200.0 / 25.4) as i32 };
    let landscape = if matches!(ps.orientation, tench_document_core::Orientation::Landscape) {
        "LANDSCAPE"
    } else {
        "NARROWLY"
    };
    xml.push_str("  <hp:p>\n    <hp:run charPrIDRef=\"0\">\n");
    xml.push_str("      <hp:secPr>\n");
    xml.push_str(&format!(
        "        <hp:pagePr width=\"{}\" height=\"{}\" landscape=\"{}\">\n",
        mm_to_hwpu(w),
        mm_to_hwpu(h),
        landscape
    ));
    xml.push_str(&format!(
        "          <hp:margin left=\"{}\" right=\"{}\" top=\"{}\" bottom=\"{}\" header=\"4252\" footer=\"4252\"/>\n",
        mm_to_hwpu(ps.margins.left),
        mm_to_hwpu(ps.margins.right),
        mm_to_hwpu(ps.margins.top),
        mm_to_hwpu(ps.margins.bottom)
    ));
    xml.push_str("        </hp:pagePr>\n");
    xml.push_str("      </hp:secPr>\n");
    xml.push_str("    </hp:run>\n  </hp:p>\n");

    xml.push_str("</hs:sec>\n");
    xml
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Write an inline node to XML, preserving bold/italic/underline marks.
fn write_inline_node(xml: &mut String, inline: &InlineNode) {
    match inline {
        InlineNode::Text { text, marks } => {
            let escaped = xml_escape(text);
            if marks.bold || marks.italic || marks.underline {
                let mut attrs = String::new();
                if marks.bold {
                    attrs.push_str(" bold=\"1\"");
                }
                if marks.italic {
                    attrs.push_str(" italic=\"1\"");
                }
                if marks.underline {
                    attrs.push_str(" underline=\"1\"");
                }
                if let Some(size) = marks.font_size {
                    attrs.push_str(&format!(" fontSize=\"{}\"", (size * 100.0) as i32));
                }
                if let Some(color) = &marks.text_color {
                    attrs.push_str(&format!(" textColor=\"{}\"", xml_escape(color)));
                }
                xml.push_str(&format!("      <hp:t{}>{}</hp:t>\n", attrs, escaped));
            } else {
                xml.push_str(&format!("      <hp:t>{}</hp:t>\n", escaped));
            }
        }
        InlineNode::HardBreak => {
            xml.push_str("      <hp:br type=\"hard\"/>\n");
        }
        InlineNode::Link {
            href, text, marks, ..
        } => {
            let escaped = xml_escape(text);
            let mut attrs = format!(" href=\"{}\"", xml_escape(href));
            if marks.bold {
                attrs.push_str(" bold=\"1\"");
            }
            if marks.italic {
                attrs.push_str(" italic=\"1\"");
            }
            if marks.underline {
                attrs.push_str(" underline=\"1\"");
            }
            xml.push_str(&format!("      <hp:t{}>{}</hp:t>\n", attrs, escaped));
        }
        InlineNode::InlineImage { source, alt, .. } => {
            let alt_text = alt.as_deref().unwrap_or("");
            match source {
                ImageSource::Embedded { data } => {
                    let b64 = base64_encode(data);
                    xml.push_str(&format!(
                        "      <hp:img alt=\"{}\" data=\"{}\"/>\n",
                        xml_escape(alt_text),
                        b64
                    ));
                }
                ImageSource::Referenced { path } => {
                    xml.push_str(&format!(
                        "      <hp:img alt=\"{}\" ref=\"{}\"/>\n",
                        xml_escape(alt_text),
                        xml_escape(path)
                    ));
                }
            }
        }
    }
}

/// Convert Alignment enum to HWP XML string.
fn alignment_to_hwp(align: Alignment) -> &'static str {
    match align {
        Alignment::Left => "LEFT",
        Alignment::Center => "CENTER",
        Alignment::Right => "RIGHT",
        Alignment::Justify => "JUSTIFY",
    }
}

/// Simple base64 encoding for image data.
fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        result.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            result.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(CHARS[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}

/// Extract text from a block for block quote serialization.
fn extract_block_text_for_quote(block: &BlockNode) -> Option<String> {
    match block {
        BlockNode::Paragraph { content, .. } | BlockNode::Heading { content, .. } => {
            let text: String = content
                .iter()
                .filter_map(|n| match n {
                    InlineNode::Text { text, .. } => Some(text.as_str()),
                    _ => None,
                })
                .collect();
            if text.is_empty() {
                None
            } else {
                Some(text)
            }
        }
        _ => None,
    }
}
