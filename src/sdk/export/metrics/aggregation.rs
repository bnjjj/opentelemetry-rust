//! Metrics SDK Aggregator export API
use crate::api::metrics::{Number, Result};
use std::time::SystemTime;

/// Sum returns an aggregated sum.
pub trait Sum {
    ///TODO
    fn sum(&self) -> Result<Number>;
}

/// Count returns the number of values that were aggregated.
pub trait Count {
    ///TODO
    fn count(&self) -> Result<u64>;
}

/// Min returns the minimum value over the set of values that were aggregated.
pub trait Min {
    /// TODO
    fn min(&self) -> Result<Number>;
}

/// Max returns the maximum value over the set of values that were aggregated.
pub trait Max {
    /// TODO
    fn max(&self) -> Result<Number>;
}

/// Quantile returns an exact or estimated quantile over the
/// set of values that were aggregated.
pub trait Quantile {
    /// TODO
    fn quantile(&self, q: f64) -> Result<Number>;
}

/// LastValue returns the latest value that was aggregated.
pub trait LastValue {
    /// TODO
    fn last_value(&self) -> Result<(Number, SystemTime)>;
}

/// Points returns the raw set of values that were aggregated.
pub trait Points {
    /// TODO
    fn points(&self) -> Result<Vec<Number>>;
}

/// Buckets represents histogram buckets boundaries and counts.
///
/// For a Histogram with N defined boundaries, e.g, [x, y, z].
/// There are N+1 counts: [-inf, x), [x, y), [y, z), [z, +inf]
#[derive(Debug)]
pub struct Buckets {
    /// Boundaries are floating point numbers, even when
    /// aggregating integers.
    boundaries: Vec<f64>,

    /// Counts are floating point numbers to account for
    /// the possibility of sampling which allows for
    /// non-integer count values.
    counts: Vec<f64>,
}

impl Buckets {
    /// Create new buckets
    pub fn new(boundaries: Vec<f64>, counts: Vec<f64>) -> Self {
        Buckets { boundaries, counts }
    }
}

/// Histogram returns the count of events in pre-determined buckets.
pub trait Histogram: Sum {
    /// TODO
    fn histogram(&self) -> Result<Buckets>;
}

/// MinMaxSumCount supports the Min, Max, Sum, and Count interfaces.
pub trait MinMaxSumCount: Min + Max + Sum + Count {}

/// Distribution supports the Min, Max, Sum, Count, and Quantile
/// interfaces.
pub trait Distribution: MinMaxSumCount + Quantile {}
