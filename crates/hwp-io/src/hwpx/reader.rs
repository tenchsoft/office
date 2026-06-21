use crate::error::HwpError;
use std::io::Read;
use tench_document_core::tdm::*;

pub fn read_hwpx(data: &[u8]) -> Result<TenchDocument, HwpError> {
    let mut archive =
        zip::ZipArchive::new(std::io::Cursor::new(data.to_vec())).map_err(HwpError::Zip)?;

    let header_xml = read_zip_string(&mut archive, "Contents/header.xml")?;
    let header_info = parse_header(&header_xml);

    let section_xml = read_zip_string(&mut archive, "Contents/section0.xml")?;
    let blocks = parse_section(&section_xml)?;

    let images = extract_images(&mut archive)?;

    // Replace placeholder Image blocks with actual image data
    let blocks = resolve_image_blocks(blocks, &images);

    let mut doc = TenchDocument::new("Untitled");
    doc.content = blocks;

    // Use metadata from header.xml
    if !header_info.title.is_empty() {
        doc.metadata.title = header_info.title.clone();
    }
    if !header_info.author.is_empty() {
        doc.metadata.author = Some(header_info.author.clone());
    }
    if !header_info.date.is_empty() {
        doc.metadata.created_at = Some(header_info.date.clone());
    }

    // Fallback: use first text block as title
    if doc.metadata.title.is_empty() {
        for block in &doc.content {
            if let Some(text) = block_text(block) {
                if !text.trim().is_empty() {
                    doc.metadata.title = text.trim().to_string();
                    break;
                }
            }
        }
    }

    Ok(doc)
}

fn read_zip_string(
    archive: &mut zip::ZipArchive<std::io::Cursor<Vec<u8>>>,
    path: &str,
) -> Result<String, HwpError> {
    let mut file = archive.by_name(path).map_err(HwpError::Zip)?;
    let mut content = String::new();
    file.read_to_string(&mut content).map_err(HwpError::Io)?;
    Ok(content)
}

fn parse_header(xml: &str) -> HeaderInfo {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    let mut info = HeaderInfo::default();
    let mut reader = Reader::from_str(xml);
    let mut in_title = false;
    let mut in_author = false;
    let mut in_date = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => match e.local_name().as_ref() {
                b"title" => in_title = true,
                b"author" => in_author = true,
                b"date" => in_date = true,
                b"fontface" => {
                    // Extract font name from face attribute
                    for attr in e.attributes().flatten() {
                        if attr.key.local_name().as_ref() == b"face" {
                            if let Ok(val) = std::str::from_utf8(&attr.value) {
                                info.fonts.push(val.to_string());
                            }
                        }
                    }
                }
                _ => {}
            },
            Ok(Event::Text(ref e)) => {
                if let Ok(text) = e.unescape() {
                    if in_title {
                        info.title = text.trim().to_string();
                    } else if in_author {
                        info.author = text.trim().to_string();
                    } else if in_date {
                        info.date = text.trim().to_string();
                    }
                }
            }
            Ok(Event::End(ref e)) => match e.local_name().as_ref() {
                b"title" => in_title = false,
                b"author" => in_author = false,
                b"date" => in_date = false,
                _ => {}
            },
            Ok(Event::Eof) => break,
            _ => {}
        }
    }
    info
}

fn parse_section(xml: &str) -> Result<Vec<BlockNode>, HwpError> {
    use quick_xml::events::Event;
    use quick_xml::Reader;

    let mut blocks = Vec::new();
    let mut reader = Reader::from_str(xml);
    let mut current_text = String::new();
    let mut in_text = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => match e.local_name().as_ref() {
                b"p" => {
                    current_text.clear();
                }
                b"t" => {
                    in_text = true;
                }
                b"tbl" => {
                    if let Some(table_block) = parse_table_block(&mut reader) {
                        blocks.push(table_block);
                    }
                }
                _ => {}
            },
            Ok(Event::Empty(_)) => {}
            Ok(Event::Text(ref e)) if in_text => {
                if let Ok(text) = e.unescape() {
                    current_text.push_str(&text)
                }
            }
            Ok(Event::End(ref e)) => match e.local_name().as_ref() {
                b"t" => {
                    in_text = false;
                }
                b"p" => {
                    let trimmed = current_text.trim();
                    if !trimmed.is_empty() {
                        blocks.push(BlockNode::Paragraph {
                            attrs: ParagraphAttrs::default(),
                            content: vec![InlineNode::Text {
                                text: trimmed.to_string(),
                                marks: Marks::default(),
                            }],
                        });
                    }
                }
                _ => {}
            },
            Ok(Event::Eof) => break,
            Err(e) => return Err(HwpError::Xml(format!("XML parse error: {}", e))),
            _ => {}
        }
    }

    Ok(blocks)
}

fn parse_table_block(reader: &mut quick_xml::Reader<&[u8]>) -> Option<BlockNode> {
    use quick_xml::events::Event;

    let mut rows = Vec::new();
    let mut current_row_cells = Vec::new();
    let mut current_cell_text = String::new();
    let mut in_cell_text = false;

    loop {
        match reader.read_event() {
            Ok(Event::Start(ref e)) => match e.local_name().as_ref() {
                b"tr" => {
                    current_row_cells.clear();
                }
                b"t" => {
                    in_cell_text = true;
                }
                _ => {}
            },
            Ok(Event::Text(ref e)) if in_cell_text => {
                if let Ok(text) = e.unescape() {
                    current_cell_text.push_str(&text)
                }
            }
            Ok(Event::End(ref e)) => match e.local_name().as_ref() {
                b"t" => {
                    in_cell_text = false;
                }
                b"tc" => {
                    current_row_cells.push(TableCell {
                        content: vec![BlockNode::Paragraph {
                            attrs: ParagraphAttrs::default(),
                            content: vec![InlineNode::Text {
                                text: current_cell_text.trim().to_string(),
                                marks: Marks::default(),
                            }],
                        }],
                        colspan: 1,
                        rowspan: 1,
                    });
                    current_cell_text.clear();
                }
                b"tr" if !current_row_cells.is_empty() => {
                    rows.push(TableRow {
                        cells: std::mem::take(&mut current_row_cells),
                    });
                }
                b"tbl" => {
                    return Some(BlockNode::Table { rows });
                }
                _ => {}
            },
            Ok(Event::Eof) => break,
            _ => {}
        }
    }
    None
}

fn extract_images(
    archive: &mut zip::ZipArchive<std::io::Cursor<Vec<u8>>>,
) -> Result<Vec<(String, Vec<u8>)>, HwpError> {
    let mut images = Vec::new();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(HwpError::Zip)?;
        let name = file.name().to_string();
        if name.starts_with("Contents/BinData/") {
            let mut data = Vec::new();
            file.read_to_end(&mut data).map_err(HwpError::Io)?;
            images.push((name, data));
        }
    }
    Ok(images)
}

/// Replace placeholder Image blocks (with binary item refs) with actual image data.
/// Replace placeholder Image blocks (with binary item refs) with actual image data.
fn resolve_image_blocks(blocks: Vec<BlockNode>, images: &[(String, Vec<u8>)]) -> Vec<BlockNode> {
    let mut result = Vec::with_capacity(blocks.len());
    for block in blocks {
        let block = match block {
            BlockNode::Image {
                source: ImageSource::Referenced { ref path },
                alt,
                width,
                height,
            } => {
                let resolved = images
                    .iter()
                    .find(|(name, _)| {
                        path.ends_with(name.split('/').next_back().unwrap_or(name))
                            || name.ends_with(path)
                    })
                    .map(|(_, data)| ImageSource::Embedded { data: data.clone() })
                    .unwrap_or(ImageSource::Referenced { path: path.clone() });
                BlockNode::Image {
                    source: resolved,
                    alt,
                    width,
                    height,
                }
            }
            other => other,
        };
        result.push(block);
    }
    result
}
fn block_text(block: &BlockNode) -> Option<String> {
    match block {
        BlockNode::Paragraph { content, .. } | BlockNode::Heading { content, .. } => Some(
            content
                .iter()
                .filter_map(|n| match n {
                    InlineNode::Text { text, .. } => Some(text.as_str()),
                    _ => None,
                })
                .collect(),
        ),
        _ => None,
    }
}

#[derive(Default)]
struct HeaderInfo {
    title: String,
    author: String,
    date: String,
    fonts: Vec<String>,
}
