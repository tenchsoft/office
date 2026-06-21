use super::super::state::CellData;

// ---------------------------------------------------------------------------
// 9.2 Chart render cache
// ---------------------------------------------------------------------------

/// Cached chart rendering data.
///
/// Avoids re-extracting and re-processing chart data when the underlying
/// grid has not changed since the last paint.
#[allow(dead_code)] // cache infrastructure — will be used when chart data updates are optimized
pub struct ChartRenderCache {
    /// Hash of the data rows used for chart rendering.
    data_hash: u64,
    /// Cached numeric values extracted from the grid.
    cached_values: Vec<f64>,
    /// Maximum value in the cached data.
    cached_max: f64,
}

#[allow(dead_code)] // cache infrastructure — will be used when chart data updates are optimized
impl ChartRenderCache {
    pub fn new() -> Self {
        Self {
            data_hash: 0,
            cached_values: Vec::new(),
            cached_max: 1.0,
        }
    }

    /// Compute a hash of the data rows that feed the chart.
    #[allow(dead_code)] // cache infrastructure — will be used when chart data updates are optimized
    fn compute_data_hash(grid: &[Vec<CellData>]) -> u64 {
        let mut hash: u64 = 0xcbf29ce484222325;
        // Hash rows 1+ (skip header row) columns 1+ (skip label column)
        for row in grid.iter().skip(1) {
            for cell in row.iter().skip(1) {
                for byte in cell.value.bytes() {
                    hash ^= byte as u64;
                    hash = hash.wrapping_mul(0x100000001b3);
                }
            }
        }
        hash
    }

    /// Update the cache from the grid. Returns cached values and max.
    pub fn update(&mut self, grid: &[Vec<CellData>]) -> (&[f64], f64) {
        let new_hash = Self::compute_data_hash(grid);
        if new_hash != self.data_hash {
            self.data_hash = new_hash;
            let values = grid
                .get(1)
                .map(|row| {
                    row.iter()
                        .skip(1)
                        .filter_map(|cell| cell.value.parse::<f64>().ok())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            let max = values.iter().copied().fold(1.0_f64, f64::max);
            self.cached_values = values;
            self.cached_max = max;
        }
        (&self.cached_values, self.cached_max)
    }
}

impl Default for ChartRenderCache {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Douglas-Peucker line simplification
// ---------------------------------------------------------------------------

/// Simplify a polyline using the Douglas-Peucker algorithm.
///
/// Returns a subset of points that approximate the original line within
/// `epsilon` distance. This is used for large datasets where rendering
/// every point would be wasteful.
#[allow(dead_code)] // public API — used when chart type is set to Line
pub fn douglas_peucker(points: &[(f64, f64)], epsilon: f64) -> Vec<(f64, f64)> {
    if points.len() <= 2 {
        return points.to_vec();
    }

    let first = points[0];
    let last = *points.last().unwrap();
    let mut max_dist = 0.0_f64;
    let mut max_idx = 0;

    for (i, point) in points.iter().enumerate().skip(1) {
        let dist = perpendicular_distance(*point, first, last);
        if dist > max_dist {
            max_dist = dist;
            max_idx = i;
        }
    }

    if max_dist > epsilon {
        let left = douglas_peucker(&points[..=max_idx], epsilon);
        let right = douglas_peucker(&points[max_idx..], epsilon);
        let mut result = left;
        result.pop();
        result.extend_from_slice(&right);
        result
    } else {
        vec![first, last]
    }
}

/// Compute the perpendicular distance from point `p` to the line segment
/// defined by `a` and `b`.
#[allow(dead_code)] // used by douglas_peucker which is public API
fn perpendicular_distance(p: (f64, f64), a: (f64, f64), b: (f64, f64)) -> f64 {
    let dx = b.0 - a.0;
    let dy = b.1 - a.1;
    let len_sq = dx * dx + dy * dy;
    if len_sq < f64::EPSILON {
        let ex = p.0 - a.0;
        let ey = p.1 - a.1;
        return (ex * ex + ey * ey).sqrt();
    }
    let cross = ((p.0 - a.0) * dy - (p.1 - a.1) * dx).abs();
    cross / len_sq.sqrt()
}
