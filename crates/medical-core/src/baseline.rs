//! Individual baseline and deviation status for one indicator (Module C).
//!
//! The baseline is the person's own average over an earlier window of values,
//! not a population norm. The system never changes a baseline by itself: a
//! persistent change only produces a *proposal* that a specialist must approve.
//!
//! Distinctions kept apart (spec Module C):
//! * reference range      - population range; none exists for 0-10 self-ratings;
//! * individual baseline  - this person's own usual values (computed here);
//! * temporary deviation  - the latest value is outside the baseline band;
//! * persistent change    - several latest values in a row are outside, same side;
//! * clinically concerning finding - only a specialist can record that.
//!
//! Window sizes and the band width are PROTOTYPE ASSUMPTIONS.

use serde::Serialize;

/// Most recent values treated as "recent", excluded from the baseline window.
pub const RECENT_WINDOW: usize = 3;
/// Older values used for the baseline (after skipping the recent window).
pub const BASELINE_SPAN: usize = 14;
/// Minimum baseline values before any comparison is made.
pub const MIN_BASELINE_VALUES: usize = 5;
/// Distance from the baseline mean, in rating points, that counts as a deviation.
pub const BAND_POINTS: f64 = 2.0;
/// Consecutive deviating values (same side) that make a persistent change.
pub const PERSISTENT_COUNT: usize = 3;
pub const BASIS: &str = "prototype_assumption";

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Baseline {
    pub mean: f64,
    pub low: f64,
    pub high: f64,
    pub values_used: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    Below,
    Above,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum BaselineStatus {
    /// Not enough history to compare.
    InsufficientData,
    WithinBaseline,
    TemporaryDeviation { direction: Direction },
    /// Proposes a baseline review; never applied automatically.
    PersistentChange { direction: Direction, count: usize },
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct IndicatorTrend {
    pub baseline: Option<Baseline>,
    pub status: BaselineStatus,
    pub latest: Option<f64>,
    /// Always false for self-ratings: there is no population reference range.
    pub has_reference_range: bool,
    pub basis: &'static str,
}

/// Baseline from values ordered NEWEST FIRST: skips the recent window and
/// averages up to `BASELINE_SPAN` older values.
pub fn compute_baseline(newest_first: &[f64]) -> Option<Baseline> {
    let older: Vec<f64> = newest_first.iter().skip(RECENT_WINDOW).take(BASELINE_SPAN).copied().collect();
    if older.len() < MIN_BASELINE_VALUES {
        return None;
    }
    let mean = older.iter().sum::<f64>() / older.len() as f64;
    Some(Baseline { mean, low: mean - BAND_POINTS, high: mean + BAND_POINTS, values_used: older.len() })
}

fn side(value: f64, b: &Baseline) -> Option<Direction> {
    if value < b.low {
        Some(Direction::Below)
    } else if value > b.high {
        Some(Direction::Above)
    } else {
        None
    }
}

/// Status of the latest values against the person's own baseline.
pub fn assess_trend(newest_first: &[f64]) -> IndicatorTrend {
    let latest = newest_first.first().copied();
    let baseline = compute_baseline(newest_first);
    let status = match (baseline, latest) {
        (Some(b), Some(latest_value)) => match side(latest_value, &b) {
            None => BaselineStatus::WithinBaseline,
            Some(direction) => {
                let count = newest_first
                    .iter()
                    .take_while(|v| side(**v, &b) == Some(direction))
                    .count();
                if count >= PERSISTENT_COUNT {
                    BaselineStatus::PersistentChange { direction, count }
                } else {
                    BaselineStatus::TemporaryDeviation { direction }
                }
            }
        },
        _ => BaselineStatus::InsufficientData,
    };
    IndicatorTrend { baseline, status, latest, has_reference_range: false, basis: BASIS }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// newest first: `recent` values, then `older` repeated history.
    fn series(recent: &[f64], older: f64, older_count: usize) -> Vec<f64> {
        let mut v = recent.to_vec();
        v.extend(std::iter::repeat_n(older, older_count));
        v
    }

    #[test]
    fn too_little_history_gives_insufficient_data() {
        let t = assess_trend(&series(&[8.0, 8.0, 8.0], 8.0, MIN_BASELINE_VALUES - 1));
        assert_eq!(t.status, BaselineStatus::InsufficientData);
        assert!(t.baseline.is_none());
    }

    #[test]
    fn baseline_ignores_the_recent_window() {
        let b = compute_baseline(&series(&[1.0, 1.0, 1.0], 8.0, 10)).unwrap();
        assert_eq!(b.mean, 8.0);
        assert_eq!(b.values_used, 10);
    }

    #[test]
    fn value_inside_band_is_within_baseline() {
        let t = assess_trend(&series(&[7.0, 8.0, 8.0], 8.0, 10));
        assert_eq!(t.status, BaselineStatus::WithinBaseline);
    }

    #[test]
    fn single_low_value_is_a_temporary_deviation() {
        let t = assess_trend(&series(&[4.0, 8.0, 8.0], 8.0, 10));
        assert_eq!(t.status, BaselineStatus::TemporaryDeviation { direction: Direction::Below });
    }

    #[test]
    fn three_low_values_in_a_row_are_a_persistent_change() {
        let t = assess_trend(&series(&[4.0, 5.0, 4.0], 8.0, 10));
        assert_eq!(t.status, BaselineStatus::PersistentChange { direction: Direction::Below, count: 3 });
    }

    #[test]
    fn deviations_on_different_sides_are_not_persistent() {
        // newest below, the one before above: only the newest counts
        let mut v = vec![4.0, 10.5, 4.0];
        v.extend(std::iter::repeat_n(8.0, 10));
        let t = assess_trend(&v);
        assert_eq!(t.status, BaselineStatus::TemporaryDeviation { direction: Direction::Below });
    }

    #[test]
    fn self_ratings_never_claim_a_reference_range() {
        assert!(!assess_trend(&series(&[8.0], 8.0, 10)).has_reference_range);
    }
}
