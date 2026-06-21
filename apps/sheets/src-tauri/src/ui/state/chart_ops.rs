use super::*;

impl SheetsState {
    pub fn open_chart_wizard(&mut self) {
        let (sr, sc, er, ec) = self.selection_range();
        let range_str = if sr == er && sc == ec {
            // Default to a reasonable range
            let max_row = (self.grid.len().min(6)).saturating_sub(1);
            let max_col = self
                .grid
                .first()
                .map(|r| r.len().min(5).saturating_sub(1))
                .unwrap_or(0);
            format!(
                "{}{}:{}{}",
                col_letter(0),
                1,
                col_letter(max_col),
                max_row + 1
            )
        } else {
            format!("{}{}:{}{}", col_letter(sc), sr + 1, col_letter(ec), er + 1)
        };
        self.chart_wizard_data_range = range_str;
        self.chart_wizard_step = 0;
        self.chart_wizard_chart_type = ChartType::Bar;
        self.chart_wizard_title = String::new();
        self.chart_wizard_show_legend = true;
        self.chart_wizard_show_axis_labels = true;
        self.show_chart_wizard = true;
    }

    /// Create a chart from the wizard state and add it to the charts list.
    pub fn create_chart_from_wizard(&mut self) {
        let chart = ChartDefinition {
            data_range: self.chart_wizard_data_range.clone(),
            chart_type: self.chart_wizard_chart_type,
            title: if self.chart_wizard_title.is_empty() {
                "Chart".to_string()
            } else {
                self.chart_wizard_title.clone()
            },
            show_legend: self.chart_wizard_show_legend,
            show_axis_labels: self.chart_wizard_show_axis_labels,
        };
        self.charts.push(chart);
        self.active_chart_idx = self.charts.len().saturating_sub(1);
        self.show_chart_wizard = false;
        self.show_chart_panel = true;
    }

    /// Parse a data range string (e.g., "A1:D5") into (start_row, start_col, end_row, end_col).
    pub fn parse_chart_data_range(&self, range_str: &str) -> Option<(usize, usize, usize, usize)> {
        let parts: Vec<&str> = range_str.split(':').collect();
        if parts.len() == 1 {
            // Single cell
            let (col, row) = parse_cell_ref(parts[0])?;
            Some((row, col, row, col))
        } else if parts.len() == 2 {
            let (sc, sr) = parse_cell_ref(parts[0])?;
            let (ec, er) = parse_cell_ref(parts[1])?;
            Some((sr, sc, er, ec))
        } else {
            None
        }
    }

    /// Get cell values from a parsed chart data range as numeric values.
    /// Returns (labels, series_data) where labels are the first column and series_data are the numeric columns.
    pub fn chart_data_from_range(&self, range_str: &str) -> (Vec<String>, Vec<Vec<f64>>) {
        let Some((sr, sc, er, ec)) = self.parse_chart_data_range(range_str) else {
            return (Vec::new(), Vec::new());
        };

        let mut labels = Vec::new();
        let mut series = Vec::new();

        // First column = labels, remaining columns = data series
        let num_series = ec.saturating_sub(sc);
        for _ in 0..num_series {
            series.push(Vec::new());
        }

        for r in sr..=er {
            if let Some(row) = self.grid.get(r) {
                // Label from first column
                let label = row
                    .get(sc)
                    .map(|c| c.display().to_string())
                    .unwrap_or_default();
                labels.push(label);

                // Data from remaining columns
                for (i, c) in (sc + 1..=ec).enumerate() {
                    let val = row
                        .get(c)
                        .and_then(|cell| cell.display().parse::<f64>().ok())
                        .unwrap_or(0.0);
                    if let Some(s) = series.get_mut(i) {
                        s.push(val);
                    }
                }
            }
        }

        (labels, series)
    }

    /// Switch to the previous chart in the list.
    pub fn prev_chart(&mut self) -> bool {
        if self.charts.is_empty() || self.active_chart_idx == 0 {
            return false;
        }
        self.active_chart_idx -= 1;
        true
    }

    /// Switch to the next chart in the list.
    pub fn next_chart(&mut self) -> bool {
        if self.charts.is_empty() || self.active_chart_idx >= self.charts.len() - 1 {
            return false;
        }
        self.active_chart_idx += 1;
        true
    }

    /// Delete the current chart.
    pub fn delete_current_chart(&mut self) -> bool {
        if self.charts.is_empty() {
            return false;
        }
        self.charts.remove(self.active_chart_idx);
        if self.active_chart_idx >= self.charts.len() {
            self.active_chart_idx = self.charts.len().saturating_sub(1);
        }
        if self.charts.is_empty() {
            self.show_chart_panel = false;
        }
        true
    }
}
