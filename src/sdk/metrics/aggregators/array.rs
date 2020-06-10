use crate::api::{
    metrics::{Descriptor, MetricsError, Number, NumberKind, Result},
    Context,
};
use crate::sdk::metrics::{
    aggregators::{Count, Distribution, Max, Min, MinMaxSumCount, Quantile, Sum},
    Aggregator,
};
use std::any::Any;
use std::cmp;
use std::mem;
use std::sync::{Arc, Mutex};

/// TODO
pub fn array() -> ArrayAggregator {
    ArrayAggregator::default()
}

///TODO
#[derive(Debug, Default)]
pub struct ArrayAggregator {
    inner: Mutex<Inner>,
}

impl Min for ArrayAggregator {
    fn min(&self) -> Result<Number> {
        self.inner
            .lock()
            .map_err(Into::into)
            .and_then(|inner| inner.checkpoint.quantile(0.0))
    }
}

impl Max for ArrayAggregator {
    fn max(&self) -> Result<Number> {
        self.inner
            .lock()
            .map_err(Into::into)
            .and_then(|inner| inner.checkpoint.quantile(1.0))
    }
}

impl Sum for ArrayAggregator {
    fn sum(&self) -> Result<Number> {
        self.inner
            .lock()
            .map_err(Into::into)
            .map(|inner| inner.checkpoint_sum.clone())
    }
}

impl Count for ArrayAggregator {
    fn count(&self) -> Result<u64> {
        self.inner
            .lock()
            .map_err(Into::into)
            .map(|inner| inner.checkpoint.len() as u64)
    }
}

impl MinMaxSumCount for ArrayAggregator {}

impl Quantile for ArrayAggregator {
    fn quantile(&self, q: f64) -> Result<Number> {
        self.inner
            .lock()
            .map_err(Into::into)
            .and_then(|inner| inner.checkpoint.quantile(q))
    }
}

impl Distribution for ArrayAggregator {}

impl Aggregator for ArrayAggregator {
    fn update_with_context(
        &self,
        _cx: &Context,
        number: &Number,
        _descriptor: &Descriptor,
    ) -> Result<()> {
        self.inner
            .lock()
            .map_err(Into::into)
            .map(|mut inner| inner.current.push(number.clone()))
    }
    fn checkpoint(&self, descriptor: &Descriptor) {
        let _lock = self.inner.lock().map(|mut inner| {
            inner.checkpoint = mem::take(&mut inner.current);

            inner.checkpoint.sort(descriptor.number_kind());

            inner.checkpoint_sum =
                inner
                    .checkpoint
                    .0
                    .iter()
                    .fold(NumberKind::U64.zero(), |acc, num| {
                        acc.add(descriptor.number_kind(), num);
                        acc
                    });
        });
    }
    fn merge(&self, other: &Arc<dyn Aggregator + Send + Sync>, desc: &Descriptor) -> Result<()> {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            self.inner.lock().map_err(Into::into).and_then(|mut inner| {
                other
                    .inner
                    .lock()
                    .map_err(From::from)
                    .and_then(|other_inner| {
                        inner
                            .checkpoint_sum
                            .add(desc.number_kind(), &other_inner.checkpoint_sum);
                        inner
                            .checkpoint
                            .combine(desc.number_kind(), &other_inner.checkpoint);
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

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug, Default)]
struct Inner {
    /// checkpoint_sum needs to be aligned for 64-bit atomic operations.
    checkpoint_sum: Number,
    current: Points,
    checkpoint: Points,
}

#[derive(Debug, Default)]
struct Points(Vec<Number>);

impl Points {
    fn len(&self) -> usize {
        self.0.len()
    }

    fn push(&mut self, number: Number) {
        self.0.push(number)
    }

    fn sort(&mut self, kind: &NumberKind) {
        match kind {
            NumberKind::F64 => self.0.sort_by(|a, b| {
                // FIXME better handling of f64 nan values
                a.to_f64()
                    .partial_cmp(&b.to_f64())
                    .unwrap_or(cmp::Ordering::Less)
            }),
            NumberKind::U64 => self.0.sort_by(|a, b| a.to_u64().cmp(&b.to_u64())),
        }
    }
    fn combine(&mut self, kind: &NumberKind, other: &Points) {
        self.0.append(&mut other.0.clone());
        self.sort(kind)
    }
}

impl Quantile for Points {
    fn quantile(&self, q: f64) -> Result<Number> {
        if self.0.is_empty() {
            return Err(MetricsError::NoDataCollected);
        }

        if q < 0.0 || q > 1.0 {
            return Err(MetricsError::InvalidQuantile);
        }

        if q == 0.0 || self.0.len() == 1 {
            return Ok(self.0[0].clone());
        } else if (q - 1.0).abs() < std::f64::EPSILON {
            return Ok(self.0[self.0.len() - 1].clone());
        }

        let position = (self.0.len() as f64 - 1.0) * q;
        Ok(self.0[position.ceil() as usize].clone())
    }
}
