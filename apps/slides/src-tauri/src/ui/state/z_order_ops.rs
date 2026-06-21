use super::*;

impl SlidesState {
    pub fn bring_forward(&mut self) {
        let Some(idx) = self.selected_element else {
            return;
        };
        {
            let Some(slide) = self.current_slide() else {
                return;
            };
            if idx + 1 >= slide.elements.len() {
                return;
            }
        }
        self.push_undo();
        if let Some(slide) = self.current_slide_mut() {
            slide.elements.swap(idx, idx + 1);
        }
        self.selected_element = Some(idx + 1);
        self.sync_content_from_slides();
    }

    pub fn send_backward(&mut self) {
        let Some(idx) = self.selected_element else {
            return;
        };
        if idx == 0 {
            return;
        }
        self.push_undo();
        if let Some(slide) = self.current_slide_mut() {
            slide.elements.swap(idx, idx - 1);
        }
        self.selected_element = Some(idx - 1);
        self.sync_content_from_slides();
    }

    pub fn bring_to_front(&mut self) {
        let Some(idx) = self.selected_element else {
            return;
        };
        {
            let Some(slide) = self.current_slide() else {
                return;
            };
            if idx + 1 >= slide.elements.len() {
                return;
            }
        }
        self.push_undo();
        if let Some(slide) = self.current_slide_mut() {
            let elem = slide.elements.remove(idx);
            slide.elements.push(elem);
            self.selected_element = Some(slide.elements.len() - 1);
        }
        self.sync_content_from_slides();
    }

    pub fn send_to_back(&mut self) {
        let Some(idx) = self.selected_element else {
            return;
        };
        if idx == 0 {
            return;
        }
        self.push_undo();
        if let Some(slide) = self.current_slide_mut() {
            let elem = slide.elements.remove(idx);
            slide.elements.insert(0, elem);
        }
        self.selected_element = Some(0);
        self.sync_content_from_slides();
    }

    // ── Phase 1.7: Grouping ─────────────────────────────────────────

    pub fn group_selected(&mut self) {
        if self.selected_elements.len() < 2 {
            return;
        }
        self.push_undo();
        let group_id = format!("group_{}", self.next_group_id);
        self.next_group_id += 1;
        let indices = self.selected_elements.clone();
        for idx in &indices {
            if let Some(slide) = self.current_slide_mut() {
                if let Some(elem) = slide.elements.get_mut(*idx) {
                    elem.group_id = Some(group_id.clone());
                }
            }
        }
        self.sync_content_from_slides();
    }

    pub fn ungroup_selected(&mut self) {
        let Some(idx) = self.selected_element else {
            return;
        };
        let group_id = {
            let Some(slide) = self.current_slide() else {
                return;
            };
            let Some(elem) = slide.elements.get(idx) else {
                return;
            };
            match &elem.group_id {
                Some(g) => g.clone(),
                None => return,
            }
        };
        self.push_undo();
        let count = self.slides.len();
        for si in 0..count {
            for elem in &mut self.slides[si].elements {
                if elem.group_id.as_deref() == Some(&group_id) {
                    elem.group_id = None;
                }
            }
        }
        self.selected_elements.clear();
        self.sync_content_from_slides();
    }
}
