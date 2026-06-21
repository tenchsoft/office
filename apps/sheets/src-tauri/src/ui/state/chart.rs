// ---------------------------------------------------------------------------
// Phase 7: Chart types
// ---------------------------------------------------------------------------

/// Chart type for the chart wizard and panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ChartType {
    #[default]
    Bar,
    Line,
    Pie,
    Area,
    Scatter,
}

impl ChartType {
    pub fn label(&self) -> &str {
        match self {
            ChartType::Bar => "Bar",
            ChartType::Line => "Line",
            ChartType::Pie => "Pie",
            ChartType::Area => "Area",
            ChartType::Scatter => "Scatter",
        }
    }

    pub const ALL: [ChartType; 5] = [
        ChartType::Bar,
        ChartType::Line,
        ChartType::Pie,
        ChartType::Area,
        ChartType::Scatter,
    ];
}

/// A chart definition created from the chart wizard.
#[derive(Debug, Clone)]
pub struct ChartDefinition {
    pub data_range: String,
    pub chart_type: ChartType,
    pub title: String,
    pub show_legend: bool,
    pub show_axis_labels: bool,
}

impl Default for ChartDefinition {
    fn default() -> Self {
        Self {
            data_range: String::new(),
            chart_type: ChartType::Bar,
            title: String::new(),
            show_legend: true,
            show_axis_labels: true,
        }
    }
}
