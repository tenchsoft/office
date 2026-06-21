use super::*;

impl DocumentEngine {
    // ----- page setup -----

    /// Update the entire page setup of the document.
    pub fn set_page_setup(&mut self, page_setup: PageSetup) -> EditResult {
        self.push_undo();
        self.document.page_setup = page_setup;
        self.dirty = true;
        self.make_result()
    }

    /// Change the paper size.
    pub fn set_paper_size(&mut self, paper_size: PaperSize) -> EditResult {
        self.push_undo();
        self.document.page_setup.paper_size = paper_size;
        self.dirty = true;
        self.make_result()
    }

    /// Change the page orientation.
    pub fn set_orientation(&mut self, orientation: Orientation) -> EditResult {
        self.push_undo();
        self.document.page_setup.orientation = orientation;
        self.dirty = true;
        self.make_result()
    }

    /// Update all four margins at once.
    pub fn set_margins(&mut self, margins: Margins) -> EditResult {
        self.push_undo();
        self.document.page_setup.margins = margins;
        self.dirty = true;
        self.make_result()
    }

    // ----- headers/footers -----

    /// Update the entire headers/footers configuration.
    pub fn set_headers_footers(&mut self, hf: HeadersFooters) -> EditResult {
        self.push_undo();
        self.document.headers_footers = hf;
        self.dirty = true;
        self.make_result()
    }

    /// Set the default header template.
    pub fn set_default_header(&mut self, template: String) -> EditResult {
        self.push_undo();
        self.document.headers_footers.default_header = Some(template);
        self.dirty = true;
        self.make_result()
    }

    /// Set the default footer template.
    pub fn set_default_footer(&mut self, template: String) -> EditResult {
        self.push_undo();
        self.document.headers_footers.default_footer = Some(template);
        self.dirty = true;
        self.make_result()
    }
}
