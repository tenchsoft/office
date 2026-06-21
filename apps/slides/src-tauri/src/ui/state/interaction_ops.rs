use super::*;

impl SlidesState {
    pub fn begin_drag(&mut self, element_idx: usize, start_pos: Point) {
        self.push_undo();
        let origins = if let Some(slide) = self.current_slide() {
            let mut v = Vec::new();
            if self.selected_elements.contains(&element_idx) {
                for &idx in &self.selected_elements {
                    if let Some(e) = slide.elements.get(idx) {
                        v.push((idx, e.x, e.y, e.w, e.h));
                    }
                }
            } else {
                if let Some(e) = slide.elements.get(element_idx) {
                    v.push((element_idx, e.x, e.y, e.w, e.h));
                }
            }
            v
        } else {
            Vec::new()
        };
        self.interaction = CanvasInteraction {
            mode: DragMode::Move,
            start_pos,
            element_origins: origins,
            box_select_origin: None,
        };
    }

    pub fn update_drag(&mut self, current_pos: Point) {
        let dx = current_pos.x - self.interaction.start_pos.x;
        let dy = current_pos.y - self.interaction.start_pos.y;
        let origins: Vec<(usize, f64, f64)> = self
            .interaction
            .element_origins
            .iter()
            .map(|(idx, ox, oy, _, _)| (*idx, *ox, *oy))
            .collect();
        for (idx, ox, oy) in origins {
            if let Some(slide) = self.current_slide_mut() {
                if let Some(elem) = slide.elements.get_mut(idx) {
                    elem.x = ox + dx;
                    elem.y = oy + dy;
                }
            }
        }
    }

    // ── Phase 1.2: Resize handles ───────────────────────────────────

    pub fn begin_resize(&mut self, element_idx: usize, handle: ResizeHandle, start_pos: Point) {
        self.push_undo();
        let origins = if let Some(slide) = self.current_slide() {
            if let Some(e) = slide.elements.get(element_idx) {
                vec![(element_idx, e.x, e.y, e.w, e.h)]
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };
        self.interaction = CanvasInteraction {
            mode: DragMode::Resize(handle),
            start_pos,
            element_origins: origins,
            box_select_origin: None,
        };
    }

    pub fn update_resize(&mut self, current_pos: Point) {
        let dx = current_pos.x - self.interaction.start_pos.x;
        let dy = current_pos.y - self.interaction.start_pos.y;
        let Some(&(idx, ox, oy, ow, oh)) = self.interaction.element_origins.first() else {
            return;
        };
        let (new_x, new_y, new_w, new_h) = match self.interaction.mode {
            DragMode::Resize(handle) => match handle {
                ResizeHandle::TopLeft => (ox + dx, oy + dy, ow - dx, oh - dy),
                ResizeHandle::TopCenter => (ox, oy + dy, ow, oh - dy),
                ResizeHandle::TopRight => (ox, oy + dy, ow + dx, oh - dy),
                ResizeHandle::MiddleLeft => (ox + dx, oy, ow - dx, oh),
                ResizeHandle::MiddleRight => (ox, oy, ow + dx, oh),
                ResizeHandle::BottomLeft => (ox + dx, oy, ow - dx, oh + dy),
                ResizeHandle::BottomCenter => (ox, oy, ow, oh + dy),
                ResizeHandle::BottomRight => (ox, oy, ow + dx, oh + dy),
            },
            _ => return,
        };
        let min_size = 10.0;
        if let Some(slide) = self.current_slide_mut() {
            if let Some(elem) = slide.elements.get_mut(idx) {
                elem.x = new_x;
                elem.y = new_y;
                elem.w = new_w.max(min_size);
                elem.h = new_h.max(min_size);
            }
        }
    }

    // ── Phase 1.3: Rotation handle ──────────────────────────────────

    pub fn begin_rotate(&mut self, element_idx: usize, start_pos: Point) {
        self.push_undo();
        let origins = if let Some(slide) = self.current_slide() {
            if let Some(e) = slide.elements.get(element_idx) {
                vec![(element_idx, e.x, e.y, e.w, e.h)]
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };
        self.interaction = CanvasInteraction {
            mode: DragMode::Rotate,
            start_pos,
            element_origins: origins,
            box_select_origin: None,
        };
    }

    pub fn update_rotate(&mut self, current_pos: Point) {
        let Some(&(idx, ox, oy, ow, oh)) = self.interaction.element_origins.first() else {
            return;
        };
        let cx = ox + ow / 2.0;
        let cy = oy + oh / 2.0;
        let start_angle =
            (self.interaction.start_pos.y - cy).atan2(self.interaction.start_pos.x - cx);
        let current_angle = (current_pos.y - cy).atan2(current_pos.x - cx);
        let delta_degrees = (current_angle - start_angle).to_degrees();
        if let Some(slide) = self.current_slide_mut() {
            if let Some(elem) = slide.elements.get_mut(idx) {
                elem.rotation += delta_degrees;
            }
        }
        self.interaction.start_pos = current_pos;
    }

    // ── Phase 1.4: Multi-select / Box select ────────────────────────

    pub fn begin_box_select(&mut self, start_pos: Point) {
        self.interaction = CanvasInteraction {
            mode: DragMode::BoxSelect,
            start_pos,
            element_origins: Vec::new(),
            box_select_origin: Some(start_pos),
        };
    }

    pub fn update_box_select(&mut self, current_pos: Point) {
        let origin = match self.interaction.box_select_origin {
            Some(p) => p,
            None => return,
        };
        let sel_rect = Rect::from_points(origin, current_pos);
        if let Some(slide) = self.current_slide() {
            self.selected_elements = slide
                .elements
                .iter()
                .enumerate()
                .filter(|(_, e)| {
                    let e_rect = Rect::new(e.x, e.y, e.x + e.w, e.y + e.h);
                    sel_rect.intersect(e_rect).area() > 0.0
                })
                .map(|(i, _)| i)
                .collect();
            self.selected_element = self.selected_elements.first().copied();
        }
    }

    pub fn toggle_element_selection(&mut self, idx: usize) {
        if self.selected_elements.contains(&idx) {
            self.selected_elements.retain(|&i| i != idx);
        } else {
            self.selected_elements.push(idx);
        }
        self.selected_element = Some(idx);
    }

    pub fn end_interaction(&mut self) {
        self.interaction = CanvasInteraction::default();
        self.sync_content_from_slides();
    }

    // ── Phase 1.5: Align/distribute ─────────────────────────────────

    pub fn align_selected(&mut self, alignment: Alignment) {
        if self.selected_elements.is_empty() {
            return;
        }
        self.push_undo();
        let indices: Vec<usize> = self.selected_elements.clone();
        match alignment {
            Alignment::Left => {
                if let Some(min_x) = self.get_elements_min_x(&indices) {
                    for idx in &indices {
                        if let Some(slide) = self.current_slide_mut() {
                            if let Some(elem) = slide.elements.get_mut(*idx) {
                                elem.x = min_x;
                            }
                        }
                    }
                }
            }
            Alignment::Right => {
                if let Some(max_x) = self.get_elements_max_right(&indices) {
                    for idx in &indices {
                        if let Some(slide) = self.current_slide_mut() {
                            if let Some(elem) = slide.elements.get_mut(*idx) {
                                elem.x = max_x - elem.w;
                            }
                        }
                    }
                }
            }
            Alignment::Top => {
                if let Some(min_y) = self.get_elements_min_y(&indices) {
                    for idx in &indices {
                        if let Some(slide) = self.current_slide_mut() {
                            if let Some(elem) = slide.elements.get_mut(*idx) {
                                elem.y = min_y;
                            }
                        }
                    }
                }
            }
            Alignment::Bottom => {
                if let Some(max_y) = self.get_elements_max_bottom(&indices) {
                    for idx in &indices {
                        if let Some(slide) = self.current_slide_mut() {
                            if let Some(elem) = slide.elements.get_mut(*idx) {
                                elem.y = max_y - elem.h;
                            }
                        }
                    }
                }
            }
            Alignment::CenterH => {
                if let (Some(min_x), Some(max_x)) = (
                    self.get_elements_min_x(&indices),
                    self.get_elements_max_right(&indices),
                ) {
                    let center = (min_x + max_x) / 2.0;
                    for idx in &indices {
                        if let Some(slide) = self.current_slide_mut() {
                            if let Some(elem) = slide.elements.get_mut(*idx) {
                                elem.x = center - elem.w / 2.0;
                            }
                        }
                    }
                }
            }
            Alignment::CenterV => {
                if let (Some(min_y), Some(max_y)) = (
                    self.get_elements_min_y(&indices),
                    self.get_elements_max_bottom(&indices),
                ) {
                    let center = (min_y + max_y) / 2.0;
                    for idx in &indices {
                        if let Some(slide) = self.current_slide_mut() {
                            if let Some(elem) = slide.elements.get_mut(*idx) {
                                elem.y = center - elem.h / 2.0;
                            }
                        }
                    }
                }
            }
            Alignment::DistributeH => {
                self.distribute_horizontal(&indices);
            }
            Alignment::DistributeV => {
                self.distribute_vertical(&indices);
            }
        }
        self.sync_content_from_slides();
    }

    fn get_elements_min_x(&self, indices: &[usize]) -> Option<f64> {
        self.with_elements(indices)
            .map(|elems| elems.iter().map(|e| e.x).fold(f64::MAX, f64::min))
    }

    fn get_elements_max_right(&self, indices: &[usize]) -> Option<f64> {
        self.with_elements(indices)
            .map(|elems| elems.iter().map(|e| e.x + e.w).fold(f64::MIN, f64::max))
    }

    fn get_elements_min_y(&self, indices: &[usize]) -> Option<f64> {
        self.with_elements(indices)
            .map(|elems| elems.iter().map(|e| e.y).fold(f64::MAX, f64::min))
    }

    fn get_elements_max_bottom(&self, indices: &[usize]) -> Option<f64> {
        self.with_elements(indices)
            .map(|elems| elems.iter().map(|e| e.y + e.h).fold(f64::MIN, f64::max))
    }

    fn with_elements(&self, indices: &[usize]) -> Option<Vec<&SlideElement>> {
        let slide = self.current_slide()?;
        Some(
            indices
                .iter()
                .filter_map(|&i| slide.elements.get(i))
                .collect(),
        )
    }

    fn distribute_horizontal(&mut self, indices: &[usize]) {
        if indices.len() < 3 {
            return;
        }
        let slide = match self.current_slide() {
            Some(s) => s,
            None => return,
        };
        let mut items: Vec<(usize, f64, f64)> = indices
            .iter()
            .filter_map(|&i| slide.elements.get(i).map(|e| (i, e.x, e.w)))
            .collect();
        items.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        let first_right = items.first().map(|i| i.1 + i.2).unwrap_or(0.0);
        let last_left = items.last().map(|i| i.1).unwrap_or(0.0);
        let total_w: f64 = items.iter().map(|i| i.2).sum();
        let gap_count = items.len().saturating_sub(1) as f64;
        let gap = if gap_count > 0.0 {
            (last_left - first_right - total_w + items.first().unwrap().2 + items.last().unwrap().2)
                / gap_count
        } else {
            0.0
        };
        let _ = slide;
        let mut x = items.first().map(|i| i.1).unwrap_or(0.0);
        for (idx, _, w) in &items {
            if let Some(slide) = self.current_slide_mut() {
                if let Some(elem) = slide.elements.get_mut(*idx) {
                    elem.x = x;
                    x += w + gap;
                }
            }
        }
    }

    fn distribute_vertical(&mut self, indices: &[usize]) {
        if indices.len() < 3 {
            return;
        }
        let slide = match self.current_slide() {
            Some(s) => s,
            None => return,
        };
        let mut items: Vec<(usize, f64, f64)> = indices
            .iter()
            .filter_map(|&i| slide.elements.get(i).map(|e| (i, e.y, e.h)))
            .collect();
        items.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        let first_bottom = items.first().map(|i| i.1 + i.2).unwrap_or(0.0);
        let last_top = items.last().map(|i| i.1).unwrap_or(0.0);
        let total_h: f64 = items.iter().map(|i| i.2).sum();
        let gap_count = items.len().saturating_sub(1) as f64;
        let gap = if gap_count > 0.0 {
            (last_top - first_bottom - total_h + items.first().unwrap().2 + items.last().unwrap().2)
                / gap_count
        } else {
            0.0
        };
        let _ = slide;
        let mut y = items.first().map(|i| i.1).unwrap_or(0.0);
        for (idx, _, h) in &items {
            if let Some(slide) = self.current_slide_mut() {
                if let Some(elem) = slide.elements.get_mut(*idx) {
                    elem.y = y;
                    y += h + gap;
                }
            }
        }
    }
}
