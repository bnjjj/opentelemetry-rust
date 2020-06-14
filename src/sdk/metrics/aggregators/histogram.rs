use crate::api::{
    metrics::{Descriptor, MetricsError, Number, NumberKind, Result},
    Context,
};
use crate::sdk::export::metrics::{Buckets, Count, Histogram, Sum};
use crate::sdk::metrics::export::metrics::Aggregator;
use std::mem;
use std::sync::{Arc, Mutex};

/// Create a new histogram for the given descriptor with the given boundaries
pub fn histogram(desc: &Descriptor, boundaries: &[f64]) -> HistogramAggregator {
    let mut sorted_boundaries = boundaries.to_owned();
    sorted_boundaries.sort_by(|a, b| a.partial_cmp(&b).unwrap());
    let state = State::empty(&sorted_boundaries);

    HistogramAggregator {
        inner: Mutex::new(Inner {
            boundaries: sorted_boundaries,
            kind: desc.number_kind().clone(),
            state,
        }),
    }
}

/// This aggregator observes events and counts them in pre-determined buckets. It
/// also calculates the sum and count of all events.
#[derive(Debug)]
pub struct HistogramAggregator {
    inner: Mutex<Inner>,
}

#[derive(Debug)]
struct Inner {
    boundaries: Vec<f64>,
    kind: NumberKind,
    state: State,
}

#[derive(Debug)]
struct State {
    bucket_counts: Vec<f64>,
    count: Number,
    sum: Number,
}

impl State {
    fn empty(boundaries: &[f64]) -> Self {
        State {
            bucket_counts: Vec::with_capacity(boundaries.len() + 1),
            count: NumberKind::U64.zero(),
            sum: NumberKind::U64.zero(),
        }
    }
}

impl Sum for HistogramAggregator {
    fn sum(&self) -> Result<Number> {
        self.inner
            .lock()
            .map_err(From::from)
            .map(|inner| inner.state.sum.clone())
    }
}
impl Count for HistogramAggregator {
    fn count(&self) -> Result<u64> {
        self.inner
            .lock()
            .map_err(From::from)
            .map(|inner| inner.state.sum.to_u64(&NumberKind::U64))
    }
}
impl Histogram for HistogramAggregator {
    fn histogram(&self) -> Result<Buckets> {
        self.inner
            .lock()
            .map_err(From::from)
            .map(|inner| Buckets::new(inner.boundaries.clone(), inner.state.bucket_counts.clone()))
    }
}

impl Aggregator for HistogramAggregator {
    fn update_with_context(
        &self,
        _cx: &Context,
        number: &Number,
        descriptor: &Descriptor,
    ) -> Result<()> {
        self.inner.lock().map_err(From::from).and_then(|mut inner| {
            let kind = descriptor.number_kind();
            let as_float = number.to_f64(kind);

            let mut bucket_id = inner.boundaries.len();
            for (idx, boundary) in inner.boundaries.iter().enumerate() {
                if as_float < *boundary {
                    bucket_id = idx;
                    break;
                }
            }
            // Note: Binary-search was compared using the benchmarks. The following
            // code is equivalent to the linear search above:
            //
            //     bucketID := sort.Search(len(c.boundaries), func(i int) bool {
            //         return asFloat < c.boundaries[i]
            //     })
            //
            // The binary search wins for very large boundary sets, but
            // the linear search performs better up through arrays between
            // 256 and 512 elements, which is a relatively large histogram, so we
            // continue to prefer linear search.

            inner
                .state
                .count
                .saturating_add(&NumberKind::U64, &1u64.into());
            inner.state.sum.saturating_add(kind, number);
            inner.state.bucket_counts[bucket_id] += 1.0;

            Ok(())
        })
    }

    fn synchronized_copy(
        &self,
        other: &Arc<dyn Aggregator + Send + Sync>,
        _descriptor: &crate::api::metrics::Descriptor,
    ) -> Result<()> {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            self.inner.lock().map_err(From::from).and_then(|mut inner| {
                other.inner.lock().map_err(From::from).map(|mut other| {
                    let empty = State::empty(&inner.boundaries);
                    other.state = mem::replace(&mut inner.state, empty)
                })
            })
        } else {
            Err(MetricsError::InconsistentAggregator(format!(
                "Expected {:?}, got: {:?}",
                self, other
            )))
        }
    }

    fn merge(&self, other: &Arc<dyn Aggregator + Send + Sync>, desc: &Descriptor) -> Result<()> {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            self.inner.lock().map_err(From::from).and_then(|mut inner| {
                other.inner.lock().map_err(From::from).and_then(|other| {
                    inner
                        .state
                        .sum
                        .saturating_add(desc.number_kind(), &other.state.sum);
                    inner
                        .state
                        .count
                        .saturating_add(&NumberKind::U64, &other.state.count);

                    for idx in 0..inner.state.bucket_counts.len() {
                        inner.state.bucket_counts[idx] += other.state.bucket_counts[idx];
                    }

                    Ok(())
                })
            })
        } else {
            Err(MetricsError::InconsistentAggregator(format!(
                "Expected {:?}, got: {:?}",
                self, other
            )))
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
