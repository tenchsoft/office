//! Animation utilities.

/// Tracks elapsed time for animation intervals.
///
/// Call [`AnimInterval::update`] with the current timestamp on each
/// `WindowEvent::AnimFrame`. The struct accumulates elapsed time and
/// reports how many intervals have elapsed since the last call, making
/// it easy to drive fixed-timestep animations.
///
/// # Example
///
/// ```ignore
/// use tench_ui::anim::AnimInterval;
///
/// let mut timer = AnimInterval::new(16.0); // ~60 fps
/// // inside on_window_event(AnimFrame(ts)):
/// let ticks = timer.update(ts);
/// for _ in 0..ticks {
///     // advance animation by one step
/// }
/// ```
#[derive(Debug, Clone)]
pub struct AnimInterval {
    /// Interval duration in the same units as the AnimFrame timestamp (ms).
    pub interval_ms: f64,
    /// Timestamp of the last update.
    last_ts: Option<f64>,
    /// Accumulated remainder from previous updates.
    accumulator: f64,
}

impl AnimInterval {
    /// Creates a new interval tracker.
    ///
    /// `interval_ms` is the desired tick duration in milliseconds
    /// (e.g. `16.667` for 60 fps).
    pub fn new(interval_ms: f64) -> Self {
        Self {
            interval_ms,
            last_ts: None,
            accumulator: 0.0,
        }
    }

    /// Feed a new timestamp and return the number of complete intervals
    /// that have elapsed.
    ///
    /// On the very first call the timer initialises (returns 0) so that
    /// a large initial delta is not misinterpreted as many elapsed ticks.
    pub fn update(&mut self, ts: u64) -> u32 {
        let ts_f64 = ts as f64;

        let Some(last) = self.last_ts else {
            self.last_ts = Some(ts_f64);
            return 0;
        };

        let delta = ts_f64 - last;
        self.last_ts = Some(ts_f64);
        self.accumulator += delta;

        let ticks = (self.accumulator / self.interval_ms).floor() as u32;
        self.accumulator -= ticks as f64 * self.interval_ms;
        ticks
    }

    /// Resets the interval tracker to its initial state.
    pub fn reset(&mut self) {
        self.last_ts = None;
        self.accumulator = 0.0;
    }

    /// Returns the accumulated sub-interval remainder in milliseconds.
    pub fn remainder_ms(&self) -> f64 {
        self.accumulator
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anim_interval_first_update_returns_zero() {
        let mut interval = AnimInterval::new(16.0);
        assert_eq!(interval.update(1000), 0);
    }

    #[test]
    fn anim_interval_counts_ticks() {
        let mut interval = AnimInterval::new(10.0);
        interval.update(0);
        assert_eq!(interval.update(25), 2);
        assert_eq!(interval.update(35), 1);
    }

    #[test]
    fn anim_interval_accumulates_remainder() {
        let mut interval = AnimInterval::new(10.0);
        interval.update(0);
        assert_eq!(interval.update(15), 1);
        assert!((interval.remainder_ms() - 5.0).abs() < 0.001);
    }

    #[test]
    fn anim_interval_reset() {
        let mut interval = AnimInterval::new(10.0);
        interval.update(100);
        interval.update(200);
        interval.reset();
        assert_eq!(interval.update(500), 0);
    }
}
