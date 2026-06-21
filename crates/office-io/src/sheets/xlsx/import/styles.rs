use super::*;

/// Parsed style from styles.xml
pub(super) struct XlsxStyle {
    font_family: String,
    font_size: f32,
    bold: bool,
    italic: bool,
    text_color: String,
    background_color: String,
    border_top_style: String,
    border_top_color: String,
    border_bottom_style: String,
    border_bottom_color: String,
    border_left_style: String,
    border_left_color: String,
    border_right_style: String,
    border_right_color: String,
    alignment_h: String,
    alignment_v: String,
    wrap_text: bool,
    number_format: String,
}

pub(super) fn read_styles(archive: &mut zip::ZipArchive<std::fs::File>) -> Vec<XlsxStyle> {
    let Ok(mut file) = archive.by_name("xl/styles.xml") else {
        return Vec::new();
    };
    let mut xml = String::new();
    file.read_to_string(&mut xml).unwrap_or_default();

    // Parse fonts
    let fonts = parse_xlsx_fonts(&xml);
    // Parse fills
    let fills = parse_xlsx_fills(&xml);
    // Parse borders
    let borders = parse_xlsx_borders(&xml);
    // Parse number formats
    let numfmts = parse_xlsx_numfmts(&xml);
    // Parse cellXfs
    parse_xlsx_cellxfs(&xml, &fonts, &fills, &borders, &numfmts)
}

fn parse_xlsx_fonts(xml: &str) -> Vec<XlsxStyle> {
    let mut fonts = Vec::new();
    let mut in_fonts = false;
    let mut current = XlsxStyle::default_font();

    for token in xml.split('<') {
        let token = token.trim();
        if token.starts_with("fonts ") || token == "fonts>" {
            in_fonts = true;
        } else if in_fonts && token.starts_with("/fonts") {
            in_fonts = false;
        } else if in_fonts && token.starts_with("/font") {
            fonts.push(std::mem::replace(&mut current, XlsxStyle::default_font()));
        } else if in_fonts {
            if let Some(rest) = token.strip_prefix("sz ") {
                if let Some(start) = rest.find("val=\"") {
                    let v = &rest[start + 5..];
                    if let Some(end) = v.find('"') {
                        current.font_size = v[..end].parse().unwrap_or(11.0);
                    }
                }
            } else if token.starts_with("b ") || token == "b/>" || token == "b>" {
                current.bold = true;
            } else if token.starts_with("i ") || token == "i/>" || token == "i>" {
                current.italic = true;
            } else if let Some(rest) = token.strip_prefix("name ") {
                if let Some(start) = rest.find("val=\"") {
                    let v = &rest[start + 5..];
                    if let Some(end) = v.find('"') {
                        current.font_family = xml_unescape(&v[..end]);
                    }
                }
            } else if let Some(rest) = token.strip_prefix("color ") {
                if let Some(start) = rest.find("rgb=\"") {
                    let v = &rest[start + 5..];
                    if let Some(end) = v.find('"') {
                        current.text_color = v[..end].to_string();
                    }
                }
            }
        }
    }
    fonts
}

fn parse_xlsx_fills(xml: &str) -> Vec<String> {
    let mut fills = Vec::new();
    let mut in_fills = false;
    let mut current_color = String::new();
    let mut in_pattern = false;

    for token in xml.split('<') {
        let token = token.trim();
        if token.starts_with("fills ") || token == "fills>" {
            in_fills = true;
        } else if in_fills && token.starts_with("/fills") {
            in_fills = false;
        } else if in_fills && token.starts_with("/fill") {
            fills.push(std::mem::take(&mut current_color));
        } else if in_fills && token.starts_with("patternFill") {
            in_pattern = true;
            if token.contains("patternType=\"none\"") {
                current_color.clear();
            }
        } else if in_fills && in_pattern {
            if let Some(rest) = token.strip_prefix("fgColor ") {
                if let Some(start) = rest.find("rgb=\"") {
                    let v = &rest[start + 5..];
                    if let Some(end) = v.find('"') {
                        current_color = v[..end].to_string();
                    }
                }
            } else if token.starts_with("/patternFill") {
                in_pattern = false;
            }
        }
    }
    fills
}

// type_complexity: border data represented as 8-element tuple
#[allow(clippy::type_complexity)]
fn parse_xlsx_borders(
    xml: &str,
) -> Vec<(
    String,
    String,
    String,
    String,
    String,
    String,
    String,
    String,
)> {
    // Returns Vec of (top_style, top_color, bottom_style, bottom_color, left_style, left_color, right_style, right_color)
    let mut borders = Vec::new();
    let mut in_borders = false;
    let mut in_border = false;
    let mut top_s = String::new();
    let mut top_c = String::new();
    let mut bottom_s = String::new();
    let mut bottom_c = String::new();
    let mut left_s = String::new();
    let mut left_c = String::new();
    let mut right_s = String::new();
    let mut right_c = String::new();

    for token in xml.split('<') {
        let token = token.trim();
        if token.starts_with("borders ") || token == "borders>" {
            in_borders = true;
        } else if in_borders && token.starts_with("/borders") {
            in_borders = false;
        } else if in_borders && token.starts_with("border") && !token.starts_with("/border") {
            in_border = true;
            top_s.clear();
            top_c.clear();
            bottom_s.clear();
            bottom_c.clear();
            left_s.clear();
            left_c.clear();
            right_s.clear();
            right_c.clear();
        } else if in_border && token.starts_with("/border") {
            borders.push((
                std::mem::take(&mut top_s),
                std::mem::take(&mut top_c),
                std::mem::take(&mut bottom_s),
                std::mem::take(&mut bottom_c),
                std::mem::take(&mut left_s),
                std::mem::take(&mut left_c),
                std::mem::take(&mut right_s),
                std::mem::take(&mut right_c),
            ));
            in_border = false;
        } else if in_border {
            let (style, color) = parse_border_edge(token);
            if token.starts_with("top") {
                top_s = style;
                top_c = color;
            } else if token.starts_with("bottom") {
                bottom_s = style;
                bottom_c = color;
            } else if token.starts_with("left") {
                left_s = style;
                left_c = color;
            } else if token.starts_with("right") {
                right_s = style;
                right_c = color;
            }
        }
    }
    borders
}

fn parse_border_edge(token: &str) -> (String, String) {
    let mut style = String::new();
    let mut color = String::new();
    if let Some(start) = token.find("style=\"") {
        let rest = &token[start + 7..];
        if let Some(end) = rest.find('"') {
            style = rest[..end].to_string();
        }
    }
    if let Some(start) = token.find("color ") {
        let rest = &token[start..];
        if let Some(cs) = rest.find("rgb=\"") {
            let v = &rest[cs + 5..];
            if let Some(end) = v.find('"') {
                color = v[..end].to_string();
            }
        }
    }
    (style, color)
}

fn parse_xlsx_numfmts(xml: &str) -> HashMap<String, String> {
    let mut fmts = HashMap::new();
    let mut in_fmts = false;
    for token in xml.split('<') {
        let token = token.trim();
        if token.starts_with("numFmts ") || token == "numFmts>" {
            in_fmts = true;
        } else if in_fmts && token.starts_with("/numFmts") {
            in_fmts = false;
        } else if in_fmts && token.starts_with("numFmt ") {
            let id = extract_attr(token, "numFmtId");
            let code = extract_attr(token, "formatCode");
            if let (Some(id), Some(code)) = (id, code) {
                fmts.insert(id, code);
            }
        }
    }
    fmts
}

// type_complexity: border data represented as 8-element tuple
#[allow(clippy::type_complexity)]
fn parse_xlsx_cellxfs(
    xml: &str,
    fonts: &[XlsxStyle],
    fills: &[String],
    borders: &[(
        String,
        String,
        String,
        String,
        String,
        String,
        String,
        String,
    )],
    numfmts: &HashMap<String, String>,
) -> Vec<XlsxStyle> {
    let mut styles = Vec::new();
    let mut in_xfs = false;

    for token in xml.split('<') {
        let token = token.trim();
        if token.starts_with("cellXfs ") || token == "cellXfs>" {
            in_xfs = true;
        } else if in_xfs && token.starts_with("/cellXfs") {
            in_xfs = false;
        } else if in_xfs && token.starts_with("xf ") {
            let mut style = XlsxStyle::default_font();

            let font_id = extract_attr(token, "fontId")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0);
            if let Some(f) = fonts.get(font_id) {
                style.font_family = f.font_family.clone();
                style.font_size = f.font_size;
                style.bold = f.bold;
                style.italic = f.italic;
                style.text_color = f.text_color.clone();
            }

            let fill_id = extract_attr(token, "fillId")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0);
            if let Some(c) = fills.get(fill_id) {
                style.background_color = c.clone();
            }

            let border_id = extract_attr(token, "borderId")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(0);
            if let Some(b) = borders.get(border_id) {
                style.border_top_style = b.0.clone();
                style.border_top_color = b.1.clone();
                style.border_bottom_style = b.2.clone();
                style.border_bottom_color = b.3.clone();
                style.border_left_style = b.4.clone();
                style.border_left_color = b.5.clone();
                style.border_right_style = b.6.clone();
                style.border_right_color = b.7.clone();
            }

            let numfmt_id = extract_attr(token, "numFmtId").unwrap_or("0".to_string());
            if numfmt_id != "0" {
                if let Some(code) = numfmts.get(&numfmt_id) {
                    style.number_format = code.clone();
                }
            }

            // Alignment from within xf (may appear in same token or following)
            if let Some(h) = extract_attr(token, "horizontal") {
                style.alignment_h = h;
            }
            if let Some(v) = extract_attr(token, "vertical") {
                style.alignment_v = v;
            }
            if token.contains("wrapText=\"1\"") {
                style.wrap_text = true;
            }

            styles.push(style);
        }
    }
    styles
}

impl XlsxStyle {
    fn default_font() -> Self {
        XlsxStyle {
            font_family: String::new(),
            font_size: 11.0,
            bold: false,
            italic: false,
            text_color: String::new(),
            background_color: String::new(),
            border_top_style: String::new(),
            border_top_color: String::new(),
            border_bottom_style: String::new(),
            border_bottom_color: String::new(),
            border_left_style: String::new(),
            border_left_color: String::new(),
            border_right_style: String::new(),
            border_right_color: String::new(),
            alignment_h: String::new(),
            alignment_v: String::new(),
            wrap_text: false,
            number_format: String::new(),
        }
    }

    pub(super) fn to_json(&self) -> Value {
        let mut obj = serde_json::Map::new();
        if !self.font_family.is_empty() {
            obj.insert(
                "font_family".into(),
                Value::String(self.font_family.clone()),
            );
        }
        if (self.font_size - 11.0).abs() > f32::EPSILON {
            obj.insert(
                "font_size".into(),
                serde_json::Number::from_f64(self.font_size as f64)
                    .map(Value::Number)
                    .unwrap_or(Value::Null),
            );
        }
        if self.bold {
            obj.insert("bold".into(), Value::Bool(true));
        }
        if self.italic {
            obj.insert("italic".into(), Value::Bool(true));
        }
        if !self.text_color.is_empty() {
            obj.insert("text_color".into(), Value::String(self.text_color.clone()));
        }
        if !self.background_color.is_empty() {
            obj.insert(
                "background_color".into(),
                Value::String(self.background_color.clone()),
            );
        }
        if !self.border_top_style.is_empty() {
            obj.insert(
                "border_top".into(),
                Value::String(self.border_top_style.clone()),
            );
        }
        if !self.border_top_color.is_empty() {
            obj.insert(
                "border_top_color".into(),
                Value::String(self.border_top_color.clone()),
            );
        }
        if !self.border_bottom_style.is_empty() {
            obj.insert(
                "border_bottom".into(),
                Value::String(self.border_bottom_style.clone()),
            );
        }
        if !self.border_bottom_color.is_empty() {
            obj.insert(
                "border_bottom_color".into(),
                Value::String(self.border_bottom_color.clone()),
            );
        }
        if !self.border_left_style.is_empty() {
            obj.insert(
                "border_left".into(),
                Value::String(self.border_left_style.clone()),
            );
        }
        if !self.border_left_color.is_empty() {
            obj.insert(
                "border_left_color".into(),
                Value::String(self.border_left_color.clone()),
            );
        }
        if !self.border_right_style.is_empty() {
            obj.insert(
                "border_right".into(),
                Value::String(self.border_right_style.clone()),
            );
        }
        if !self.border_right_color.is_empty() {
            obj.insert(
                "border_right_color".into(),
                Value::String(self.border_right_color.clone()),
            );
        }
        if !self.alignment_h.is_empty() {
            obj.insert(
                "alignment_h".into(),
                Value::String(self.alignment_h.clone()),
            );
        }
        if !self.alignment_v.is_empty() {
            obj.insert(
                "alignment_v".into(),
                Value::String(self.alignment_v.clone()),
            );
        }
        if self.wrap_text {
            obj.insert("wrap_text".into(), Value::Bool(true));
        }
        if !self.number_format.is_empty() {
            obj.insert(
                "number_format".into(),
                Value::String(self.number_format.clone()),
            );
        }
        Value::Object(obj)
    }
}

pub(super) fn extract_attr(token: &str, attr: &str) -> Option<String> {
    let pattern = format!("{}=\"", attr);
    if let Some(start) = token.find(&pattern) {
        let rest = &token[start + pattern.len()..];
        if let Some(end) = rest.find('"') {
            return Some(xml_unescape(&rest[..end]));
        }
    }
    None
}
