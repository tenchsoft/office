use super::super::state::{ChartType, SheetsState};
use super::cache::ChartRenderCache;
use super::colors::CHART_COLORS;
use super::render::{
    paint_area_chart, paint_bar_chart, paint_line_chart, paint_pie_chart, paint_scatter_chart,
};
use tench_ui::parley::FontWeight;
use tench_ui::prelude::*;

// ---------------------------------------------------------------------------
// Paint functions
// ---------------------------------------------------------------------------

pub fn paint_chart_panel(
    state: &SheetsState,
    p: &mut Painter<'_>,
    theme: &Theme,
    rect: Rect,
    chart_cache: &mut ChartRenderCache,
) {
    p.fill_rect(rect, theme.surface);
    // Border on left side
    p.draw_line(
        Point::new(rect.x0, rect.y0),
        Point::new(rect.x0, rect.y1),
        theme.border,
        1.0,
    );

    let x = rect.x0 + 12.0;
    let mut y = rect.y0 + 20.0;

    // Resize handle (drag to resize chart panel width)
    let handle_rect = Rect::new(rect.x0 - 4.0, rect.y0, rect.x0 + 4.0, rect.y1);
    p.fill_rect(handle_rect, Color::TRANSPARENT);
    p.draw_line(
        Point::new(rect.x0, rect.y0 + rect.height() * 0.3),
        Point::new(rect.x0, rect.y0 + rect.height() * 0.7),
        theme.disabled,
        2.0,
    );

    // Header
    p.draw_text(
        "Charts",
        x,
        y,
        theme.on_surface,
        theme.font_size,
        FontWeight::BOLD,
        false,
    );

    // Prev / Next / Delete buttons (top-right)
    if state.charts.len() > 1 {
        let btn_y = rect.y0 + 6.0;
        let nav_x = rect.x1 - 100.0;
        p.draw_text(
            "<",
            nav_x,
            btn_y + 14.0,
            theme.primary,
            theme.font_size_small,
            FontWeight::BOLD,
            false,
        );
        p.draw_text(
            ">",
            nav_x + 20.0,
            btn_y + 14.0,
            theme.primary,
            theme.font_size_small,
            FontWeight::BOLD,
            false,
        );
        let idx_label = format!("{}/{}", state.active_chart_idx + 1, state.charts.len());
        p.draw_text(
            &idx_label,
            nav_x + 40.0,
            btn_y + 14.0,
            theme.secondary,
            theme.font_size_small,
            FontWeight::NORMAL,
            false,
        );
    }
    // Delete button
    if !state.charts.is_empty() {
        let del_x = rect.x1 - 30.0;
        p.draw_text(
            "X",
            del_x,
            rect.y0 + 20.0,
            Color::rgb8(0xEF, 0x44, 0x44),
            theme.font_size_small,
            FontWeight::BOLD,
            false,
        );
    }

    y += 28.0;

    if state.charts.is_empty() {
        p.draw_text(
            "No charts. Use Insert > Chart",
            x,
            y,
            theme.disabled,
            theme.font_size_small,
            FontWeight::NORMAL,
            false,
        );
        return;
    }

    let chart_def = &state.charts[state.active_chart_idx];

    // Chart title
    if !chart_def.title.is_empty() {
        p.draw_text(
            &chart_def.title,
            x,
            y,
            theme.on_surface,
            theme.font_size_small,
            FontWeight::BOLD,
            false,
        );
        y += 18.0;
    }

    // Chart area
    let chart_area = Rect::new(x, y, rect.x1 - 12.0, y + 160.0);
    p.fill_rounded_rect(chart_area, theme.background, 4.0);

    // Render the chart based on type
    match chart_def.chart_type {
        ChartType::Bar => paint_bar_chart(state, p, theme, chart_area, chart_def, chart_cache),
        ChartType::Line => paint_line_chart(state, p, theme, chart_area, chart_def, chart_cache),
        ChartType::Pie => paint_pie_chart(state, p, theme, chart_area, chart_def),
        ChartType::Area => paint_area_chart(state, p, theme, chart_area, chart_def),
        ChartType::Scatter => paint_scatter_chart(state, p, theme, chart_area, chart_def),
    }

    y += 176.0;

    // Chart type buttons
    p.draw_text(
        "Chart Type",
        x,
        y,
        theme.on_surface,
        theme.font_size,
        FontWeight::BOLD,
        false,
    );
    y += 20.0;

    for ct in ChartType::ALL {
        let is_active = ct == chart_def.chart_type;
        let label = ct.label();
        let btn_rect = Rect::new(x, y - 2.0, x + 60.0, y + 14.0);
        if is_active {
            p.fill_rounded_rect(btn_rect, theme.primary, 3.0);
            p.draw_text(
                label,
                x + 4.0,
                y + 12.0,
                Color::WHITE,
                theme.font_size_small,
                FontWeight::BOLD,
                false,
            );
        } else {
            p.fill_rounded_rect(btn_rect, theme.background, 3.0);
            p.stroke_rounded_rect(btn_rect, theme.border, 0.5, 3.0);
            p.draw_text(
                label,
                x + 4.0,
                y + 12.0,
                theme.on_surface,
                theme.font_size_small,
                FontWeight::NORMAL,
                false,
            );
        }
        y += 18.0;
    }

    // Legend
    if chart_def.show_legend {
        y += 4.0;
        let (labels, _series) = state.chart_data_from_range(&chart_def.data_range);
        if !labels.is_empty() {
            p.draw_text(
                "Legend",
                x,
                y,
                theme.on_surface,
                theme.font_size_small,
                FontWeight::BOLD,
                false,
            );
            y += 16.0;
            for (i, label) in labels.iter().enumerate() {
                if y > rect.y1 - 20.0 {
                    break;
                }
                let color = CHART_COLORS[i % CHART_COLORS.len()];
                let swatch = Rect::new(x + 4.0, y, x + 14.0, y + 10.0);
                p.fill_rounded_rect(swatch, color, 2.0);
                p.draw_text(
                    label,
                    x + 20.0,
                    y + 10.0,
                    theme.on_surface,
                    theme.font_size_small * 0.85,
                    FontWeight::NORMAL,
                    false,
                );
                y += 14.0;
            }
        }
    }
}

/// Hit-test the chart panel for navigation and chart type switching.
pub fn hit_chart_panel(
    state: &SheetsState,
    x: f64,
    y: f64,
    rect: Rect,
) -> Option<ChartPanelAction> {
    // Resize handle
    if x >= rect.x0 - 6.0 && x <= rect.x0 + 6.0 {
        return Some(ChartPanelAction::ResizeStart);
    }

    // Delete button
    if !state.charts.is_empty() {
        let del_x = rect.x1 - 30.0;
        if x >= del_x && x <= del_x + 16.0 && y >= rect.y0 + 6.0 && y <= rect.y0 + 28.0 {
            return Some(ChartPanelAction::DeleteChart);
        }
    }

    // Prev/Next buttons
    if state.charts.len() > 1 {
        let nav_x = rect.x1 - 100.0;
        let btn_y = rect.y0 + 6.0;
        if y >= btn_y && y <= btn_y + 20.0 {
            if x >= nav_x && x < nav_x + 16.0 {
                return Some(ChartPanelAction::PrevChart);
            }
            if x >= nav_x + 20.0 && x < nav_x + 36.0 {
                return Some(ChartPanelAction::NextChart);
            }
        }
    }

    // Chart type buttons
    if !state.charts.is_empty() {
        let chart_area_bottom = rect.y0 + 20.0 + 28.0 + 176.0 + 20.0;
        let mut btn_y = chart_area_bottom;
        for ct in ChartType::ALL {
            let btn_rect = Rect::new(rect.x0 + 12.0, btn_y - 2.0, rect.x0 + 72.0, btn_y + 14.0);
            if btn_rect.contains(Point::new(x, y)) {
                return Some(ChartPanelAction::SwitchChartType(ct));
            }
            btn_y += 18.0;
        }
    }

    None
}

/// Actions that can be triggered from the chart panel.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChartPanelAction {
    PrevChart,
    NextChart,
    DeleteChart,
    SwitchChartType(ChartType),
    ResizeStart,
}
