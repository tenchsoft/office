use super::*;

impl SlidesState {
    pub fn zoom_in(&mut self) {
        self.zoom.level = (self.zoom.level * 1.2).min(5.0);
    }

    pub fn zoom_out(&mut self) {
        self.zoom.level = (self.zoom.level / 1.2).max(0.2);
    }

    pub fn zoom_reset(&mut self) {
        self.zoom.level = 1.0;
        self.zoom.pan_x = 0.0;
        self.zoom.pan_y = 0.0;
    }

    pub fn pan(&mut self, dx: f64, dy: f64) {
        self.zoom.pan_x += dx;
        self.zoom.pan_y += dy;
    }

    // ── Hit testing ─────────────────────────────────────────────────

    pub fn hit_test_element(&self, point: Point, page: Rect) -> Option<usize> {
        let slide = self.current_slide()?;
        let scale_x = page.width() / 640.0;
        let scale_y = page.height() / 360.0;
        slide
            .elements
            .iter()
            .enumerate()
            .rev()
            .find_map(|(idx, elem)| {
                let rect = Rect::new(
                    page.x0 + elem.x * scale_x,
                    page.y0 + elem.y * scale_y,
                    page.x0 + (elem.x + elem.w) * scale_x,
                    page.y0 + (elem.y + elem.h) * scale_y,
                );
                rect.contains(point).then_some(idx)
            })
    }

    /// Phase 1.2: Hit test for resize handles around the selected element.
    pub fn hit_test_resize_handle(&self, point: Point, page: Rect) -> Option<ResizeHandle> {
        let idx = self.selected_element?;
        let slide = self.current_slide()?;
        let elem = slide.elements.get(idx)?;
        let scale_x = page.width() / 640.0;
        let scale_y = page.height() / 360.0;
        let ex = page.x0 + elem.x * scale_x;
        let ey = page.y0 + elem.y * scale_y;
        let ew = elem.w * scale_x;
        let eh = elem.h * scale_y;
        let handle_size = 6.0;
        let handles = [
            (ResizeHandle::TopLeft, ex, ey),
            (ResizeHandle::TopCenter, ex + ew / 2.0, ey),
            (ResizeHandle::TopRight, ex + ew, ey),
            (ResizeHandle::MiddleLeft, ex, ey + eh / 2.0),
            (ResizeHandle::MiddleRight, ex + ew, ey + eh / 2.0),
            (ResizeHandle::BottomLeft, ex, ey + eh),
            (ResizeHandle::BottomCenter, ex + ew / 2.0, ey + eh),
            (ResizeHandle::BottomRight, ex + ew, ey + eh),
        ];
        for (handle, hx, hy) in &handles {
            let hr = Rect::new(
                hx - handle_size,
                hy - handle_size,
                hx + handle_size,
                hy + handle_size,
            );
            if hr.contains(point) {
                return Some(*handle);
            }
        }
        None
    }

    /// Phase 1.3: Hit test for rotation handle above the selected element.
    pub fn hit_test_rotate_handle(&self, point: Point, page: Rect) -> bool {
        let Some(idx) = self.selected_element else {
            return false;
        };
        let Some(slide) = self.current_slide() else {
            return false;
        };
        let Some(elem) = slide.elements.get(idx) else {
            return false;
        };
        let scale_x = page.width() / 640.0;
        let scale_y = page.height() / 360.0;
        let cx = page.x0 + (elem.x + elem.w / 2.0) * scale_x;
        let top_y = page.y0 + elem.y * scale_y - 24.0;
        let hr = Rect::new(cx - 8.0, top_y - 8.0, cx + 8.0, top_y + 8.0);
        hr.contains(point)
    }
}
