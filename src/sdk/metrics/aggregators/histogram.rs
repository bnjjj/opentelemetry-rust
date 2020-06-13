use crate::api::{
    metrics::{Descriptor, MetricsError, Number, NumberKind, Result},
    Context,
};
use crate::sdk::export::metrics::{Buckets, Count, Histogram, Sum};
use crate::sdk::metrics::export::metrics::Aggregator;
use std::mem;
use std::sync::{Arc, Mutex};

/// TODO
pub fn histogram(desc: &Descriptor, boundaries: &[f64]) -> HistogramAggregator {
    let mut sorted_boundaries = boundaries.to_owned();
    sorted_boundaries.sort_by(|a, b| a.partial_cmp(&b).unwrap());

    HistogramAggregator {
        inner: Mutex::new(Inner {
            current: State::empty(&sorted_boundaries),
            checkpoint: State::empty(&sorted_boundaries),
            boundaries: sorted_boundaries,
            kind: desc.number_kind().clone(),
        }),
    }
}

/// TODO
#[derive(Debug)]
pub struct HistogramAggregator {
    inner: Mutex<Inner>,
}

#[derive(Debug)]
struct Inner {
    current: State,
    checkpoint: State,
    boundaries: Vec<f64>,
    kind: NumberKind,
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
            .map(|inner| inner.checkpoint.sum.clone())
    }
}
impl Count for HistogramAggregator {
    fn count(&self) -> Result<u64> {
        self.inner
            .lock()
            .map_err(From::from)
            .map(|inner| inner.checkpoint.sum.to_u64(&NumberKind::U64))
    }
}
impl Histogram for HistogramAggregator {
    fn histogram(&self) -> Result<Buckets> {
        self.inner.lock().map_err(From::from).map(|inner| {
            Buckets::new(
                inner.boundaries.clone(),
                inner.checkpoint.bucket_counts.clone(),
            )
        })
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

            inner.current.count.add(&NumberKind::U64, &1u64.into());
            inner.current.sum.add(kind, number);
            inner.current.bucket_counts[bucket_id] += 1.0;

            Ok(())
        })
    }

    fn checkpoint(&self, _descriptor: &crate::api::metrics::Descriptor) {
        let _lock = self.inner.lock().map(|mut inner| {
            let empty = State::empty(&inner.boundaries);
            inner.checkpoint = mem::replace(&mut inner.current, empty);
        });
    }

    fn merge(
        &self,
        other: &Arc<dyn Aggregator + Send + Sync>,
        descriptor: &Descriptor,
    ) -> Result<()> {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            self.inner.lock().map_err(From::from).and_then(|mut inner| {
                other.inner.lock().map_err(From::from).and_then(|other| {
                    inner
                        .checkpoint
                        .sum
                        .add(descriptor.number_kind(), &other.checkpoint.sum);
                    inner
                        .checkpoint
                        .count
                        .add(&NumberKind::U64, &other.checkpoint.count);

                    for idx in 0..inner.checkpoint.bucket_counts.len() {
                        inner.checkpoint.bucket_counts[idx] += other.checkpoint.bucket_counts[idx];
                    }

                    Ok(())
                })
            })
        } else {
            Err(MetricsError::InconsistentMergeError(format!(
                "Expected {:?}, got: {:?}",
                self, other
            )))
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
