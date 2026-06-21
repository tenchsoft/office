use super::super::state::{ChartDefinition, SheetsState};
use super::cache::{douglas_peucker, ChartRenderCache};
use super::colors::CHART_COLORS;
use tench_ui::parley::FontWeight;
use tench_ui::prelude::*;

// ---------------------------------------------------------------------------
// Chart painting implementations
// ---------------------------------------------------------------------------

pub(super) fn paint_bar_chart(
    state: &SheetsState,
    p: &mut Painter<'_>,
    theme: &Theme,
    rect: Rect,
    chart_def: &ChartDefinition,
    chart_cache: &mut ChartRenderCache,
) {
    let (labels, series) = state.chart_data_from_range(&chart_def.data_range);

    if series.is_empty() || labels.is_empty() {
        paint_no_data(p, theme, rect);
        return;
    }

    let base = rect.y1 - 28.0;
    let chart_height = 96.0;
    let left_margin = 12.0;

    // Find max across all series
    let max = series
        .iter()
        .flat_map(|s| s.iter().copied())
        .fold(1.0_f64, f64::max);

    let num_labels = labels.len();
    let available_width = rect.width() - left_margin * 2.0;
    let group_width = available_width / num_labels as f64;
    let num_series = series.len();
    let bar_width = (group_width * 0.7 / num_series as f64).clamp(4.0, 30.0);

    for (si, s) in series.iter().enumerate() {
        let color = CHART_COLORS[si % CHART_COLORS.len()];
        for (i, value) in s.iter().enumerate() {
            let h = (*value / max) * chart_height;
            let group_x = rect.x0 + left_margin + i as f64 * group_width;
            let bar_x = group_x
                + (group_width - bar_width * num_series as f64) / 2.0
                + si as f64 * bar_width;
            p.fill_rounded_rect(
                Rect::new(bar_x, base - h, bar_x + bar_width, base),
                color,
                2.0,
            );
        }
    }

    // X-axis labels
    if chart_def.show_axis_labels {
        for (i, label) in labels.iter().enumerate() {
            let group_x = rect.x0 + left_margin + i as f64 * group_width + group_width / 2.0;
            p.draw_text(
                label,
                group_x - 12.0,
                base + 14.0,
                theme.disabled,
                theme.font_size_small * 0.75,
                FontWeight::NORMAL,
                false,
            );
        }
    }

    // Y-axis labels
    if chart_def.show_axis_labels {
        for step in 0..=4 {
            let val = max * step as f64 / 4.0;
            let yy = base - (val / max) * chart_height;
            p.draw_text(
                &format_number(val),
                rect.x0 + 4.0,
                yy + 4.0,
                theme.disabled,
                theme.font_size_small * 0.7,
                FontWeight::NORMAL,
                false,
            );
        }
    }

    // Baseline
    p.draw_line(
        Point::new(rect.x0 + left_margin, base),
        Point::new(rect.x1 - left_margin, base),
        theme.border,
        0.5,
    );

    // Suppress unused cache warning — cache is used for legacy path
    let _ = chart_cache;
}

pub(super) fn paint_line_chart(
    state: &SheetsState,
    p: &mut Painter<'_>,
    theme: &Theme,
    rect: Rect,
    chart_def: &ChartDefinition,
    chart_cache: &mut ChartRenderCache,
) {
    let (labels, series) = state.chart_data_from_range(&chart_def.data_range);

    if series.is_empty() || labels.is_empty() {
        paint_no_data(p, theme, rect);
        return;
    }

    let base = rect.y1 - 28.0;
    let chart_height = 96.0;
    let left_margin = 12.0;

    let max = series
        .iter()
        .flat_map(|s| s.iter().copied())
        .fold(1.0_f64, f64::max);

    let num_labels = labels.len();
    let available_width = rect.width() - left_margin * 2.0;
    let step_x = if num_labels > 1 {
        available_width / (num_labels - 1) as f64
    } else {
        available_width
    };

    for (si, s) in series.iter().enumerate() {
        let color = CHART_COLORS[si % CHART_COLORS.len()];

        // Build points
        let points: Vec<(f64, f64)> = s
            .iter()
            .enumerate()
            .map(|(i, v)| {
                let px = rect.x0 + left_margin + i as f64 * step_x;
                let py = base - (*v / max) * chart_height;
                (px, py)
            })
            .collect();

        // Simplify for large datasets
        let simplified = if points.len() > 1000 {
            douglas_peucker(&points, 1.0)
        } else {
            points
        };

        // Draw polyline
        for window in simplified.windows(2) {
            let p0 = Point::new(window[0].0, window[0].1);
            let p1 = Point::new(window[1].0, window[1].1);
            p.draw_line(p0, p1, color, 2.0);
        }

        // Draw data points
        for &(px, py) in &simplified {
            p.fill_circle(Point::new(px, py), 3.0, color);
        }
    }

    // X-axis labels
    if chart_def.show_axis_labels {
        for (i, label) in labels.iter().enumerate() {
            let px = rect.x0 + left_margin + i as f64 * step_x;
            p.draw_text(
                label,
                px - 12.0,
                base + 14.0,
                theme.disabled,
                theme.font_size_small * 0.75,
                FontWeight::NORMAL,
                false,
            );
        }
    }

    // Baseline
    p.draw_line(
        Point::new(rect.x0 + left_margin, base),
        Point::new(rect.x1 - left_margin, base),
        theme.border,
        0.5,
    );

    let _ = chart_cache;
}

pub(super) fn paint_pie_chart(
    state: &SheetsState,
    p: &mut Painter<'_>,
    theme: &Theme,
    rect: Rect,
    chart_def: &ChartDefinition,
) {
    let (labels, series) = state.chart_data_from_range(&chart_def.data_range);

    if series.is_empty() || labels.is_empty() {
        paint_no_data(p, theme, rect);
        return;
    }

    // Use first series for pie chart
    let data = &series[0];
    let total: f64 = data.iter().sum();
    if total.abs() < f64::EPSILON {
        paint_no_data(p, theme, rect);
        return;
    }

    let center_x = (rect.x0 + rect.x1) / 2.0;
    let center_y = (rect.y0 + rect.y1) / 2.0 - 8.0;
    let radius = (rect.height() * 0.35).min(rect.width() * 0.35).min(65.0);

    let mut start_angle = -std::f64::consts::FRAC_PI_2; // Start from top

    for (i, &value) in data.iter().enumerate() {
        let slice_angle = (value / total) * 2.0 * std::f64::consts::PI;
        let color = CHART_COLORS[i % CHART_COLORS.len()];

        // Draw pie slice as a filled wedge (approximate with polygon)
        let num_segments = ((slice_angle / 0.1).ceil() as usize).max(2);
        let mut points = vec![Point::new(center_x, center_y)];
        for seg in 0..=num_segments {
            let angle = start_angle + slice_angle * seg as f64 / num_segments as f64;
            let px = center_x + radius * angle.cos();
            let py = center_y + radius * angle.sin();
            points.push(Point::new(px, py));
        }

        // Fill the wedge
        for j in 1..points.len() - 1 {
            let tri = [points[0], points[j], points[j + 1]];
            p.fill_triangle(tri[0], tri[1], tri[2], color);
        }

        // Label if the slice is big enough
        if slice_angle > 0.3 && i < labels.len() {
            let mid_angle = start_angle + slice_angle / 2.0;
            let label_r = radius + 14.0;
            let lx = center_x + label_r * mid_angle.cos();
            let ly = center_y + label_r * mid_angle.sin();
            p.draw_text(
                &labels[i],
                lx - 10.0,
                ly + 4.0,
                theme.on_surface,
                theme.font_size_small * 0.75,
                FontWeight::NORMAL,
                false,
            );
        }

        start_angle += slice_angle;
    }
}

pub(super) fn paint_area_chart(
    state: &SheetsState,
    p: &mut Painter<'_>,
    theme: &Theme,
    rect: Rect,
    chart_def: &ChartDefinition,
) {
    let (labels, series) = state.chart_data_from_range(&chart_def.data_range);

    if series.is_empty() || labels.is_empty() {
        paint_no_data(p, theme, rect);
        return;
    }

    let base = rect.y1 - 28.0;
    let chart_height = 96.0;
    let left_margin = 12.0;

    let max = series
        .iter()
        .flat_map(|s| s.iter().copied())
        .fold(1.0_f64, f64::max);

    let num_labels = labels.len();
    let available_width = rect.width() - left_margin * 2.0;
    let step_x = if num_labels > 1 {
        available_width / (num_labels - 1) as f64
    } else {
        available_width
    };

    for (si, s) in series.iter().enumerate() {
        let color = CHART_COLORS[si % CHART_COLORS.len()];

        // Build points for the area
        let points: Vec<Point> = s
            .iter()
            .enumerate()
            .map(|(i, v)| {
                let px = rect.x0 + left_margin + i as f64 * step_x;
                let py = base - (*v / max) * chart_height;
                Point::new(px, py)
            })
            .collect();

        if points.len() < 2 {
            continue;
        }

        // Fill area using triangles from baseline
        for i in 0..points.len() - 1 {
            let p0 = points[i];
            let p1 = points[i + 1];
            let b0 = Point::new(p0.x, base);
            let b1 = Point::new(p1.x, base);

            // Two triangles to form the quad
            let packed = color.to_u32();
            let r = ((packed >> 24) & 0xFF) as f32 / 255.0;
            let g = ((packed >> 16) & 0xFF) as f32 / 255.0;
            let b = ((packed >> 8) & 0xFF) as f32 / 255.0;
            let semi_color = Color::from_rgba(r, g, b, 0.3);
            p.fill_triangle(p0, p1, b0, semi_color);
            p.fill_triangle(b0, p1, b1, semi_color);
        }

        // Draw the line on top
        for window in points.windows(2) {
            p.draw_line(window[0], window[1], color, 2.0);
        }
    }

    // X-axis labels
    if chart_def.show_axis_labels {
        for (i, label) in labels.iter().enumerate() {
            let px = rect.x0 + left_margin + i as f64 * step_x;
            p.draw_text(
                label,
                px - 12.0,
                base + 14.0,
                theme.disabled,
                theme.font_size_small * 0.75,
                FontWeight::NORMAL,
                false,
            );
        }
    }

    // Baseline
    p.draw_line(
        Point::new(rect.x0 + left_margin, base),
        Point::new(rect.x1 - left_margin, base),
        theme.border,
        0.5,
    );
}

pub(super) fn paint_scatter_chart(
    state: &SheetsState,
    p: &mut Painter<'_>,
    theme: &Theme,
    rect: Rect,
    chart_def: &ChartDefinition,
) {
    let (labels, series) = state.chart_data_from_range(&chart_def.data_range);

    if series.len() < 2 || labels.is_empty() {
        paint_no_data(p, theme, rect);
        return;
    }

    let base = rect.y1 - 28.0;
    let chart_height = 96.0;
    let left_margin = 12.0;
    let top_margin = 12.0;

    // Use first series as X, second as Y
    let x_data = &series[0];
    let y_data = &series[1];

    let x_max = x_data.iter().copied().fold(1.0_f64, f64::max);
    let y_max = y_data.iter().copied().fold(1.0_f64, f64::max);

    let chart_width = rect.width() - left_margin * 2.0;

    // Draw grid lines
    for step in 0..=4 {
        let yy = base - (step as f64 / 4.0) * chart_height;
        p.draw_line(
            Point::new(rect.x0 + left_margin, yy),
            Point::new(rect.x1 - left_margin, yy),
            theme.border,
            0.3,
        );
    }

    // Plot points
    let count = x_data.len().min(y_data.len());
    for i in 0..count {
        let px = rect.x0 + left_margin + (x_data[i] / x_max) * chart_width;
        let py = base - (y_data[i] / y_max) * chart_height;
        let color = CHART_COLORS[i % CHART_COLORS.len()];
        p.fill_circle(Point::new(px, py), 4.0, color);

        // Label
        if i < labels.len() {
            p.draw_text(
                &labels[i],
                px + 6.0,
                py - 2.0,
                theme.disabled,
                theme.font_size_small * 0.7,
                FontWeight::NORMAL,
                false,
            );
        }
    }

    // Axis labels
    if chart_def.show_axis_labels {
        if !labels.is_empty() {
            p.draw_text(
                &labels[0],
                rect.x0 + left_margin,
                base + 14.0,
                theme.disabled,
                theme.font_size_small * 0.75,
                FontWeight::NORMAL,
                false,
            );
        }
        p.draw_text(
            &format_number(y_max),
            rect.x0 + 4.0,
            top_margin + rect.y0 + 8.0,
            theme.disabled,
            theme.font_size_small * 0.7,
            FontWeight::NORMAL,
            false,
        );
    }

    // Baseline
    p.draw_line(
        Point::new(rect.x0 + left_margin, base),
        Point::new(rect.x1 - left_margin, base),
        theme.border,
        0.5,
    );
}

fn paint_no_data(p: &mut Painter<'_>, theme: &Theme, rect: Rect) {
    p.draw_text(
        "No data in range",
        rect.x0 + 12.0,
        rect.y0 + rect.height() / 2.0,
        theme.disabled,
        theme.font_size_small,
        FontWeight::NORMAL,
        false,
    );
}

fn format_number(value: f64) -> String {
    if (value.fract()).abs() < f64::EPSILON {
        format!("{}", value as i64)
    } else {
        format!("{value:.1}")
    }
}
