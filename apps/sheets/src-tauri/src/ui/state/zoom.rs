use super::*;

impl SheetsState {
    // ----- Zoom -----

    /// Set zoom level, clamped to [25, 400].
    pub fn set_zoom(&mut self, percent: u32) -> bool {
        let clamped = percent.clamp(25, 400);
        if clamped == self.zoom_percent {
            return false;
        }
        self.zoom_percent = clamped;
        true
    }

    /// Zoom in by 10%.
    pub fn zoom_in(&mut self) -> bool {
        self.set_zoom(self.zoom_percent.saturating_add(10))
    }

    /// Zoom out by 10%.
    pub fn zoom_out(&mut self) -> bool {
        self.set_zoom(self.zoom_percent.saturating_sub(10))
    }
}
