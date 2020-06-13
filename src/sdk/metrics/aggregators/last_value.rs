use crate::api::{
    metrics::{Descriptor, MetricsError, Number, Result},
    Context,
};
use crate::sdk::export::metrics::{Aggregator, LastValue};
use std::any::Any;
use std::sync::Mutex;
use std::time::SystemTime;

/// TODO
pub fn last_value() -> LastValueAggregator {
    LastValueAggregator {
        inner: Mutex::new(Inner::default()),
    }
}

/// TODO
#[derive(Debug)]
pub struct LastValueAggregator {
    inner: Mutex<Inner>,
}

impl Aggregator for LastValueAggregator {
    fn update_with_context(
        &self,
        _cx: &Context,
        number: &Number,
        _descriptor: &Descriptor,
    ) -> Result<()> {
        self.inner.lock().map_err(Into::into).map(|mut inner| {
            inner.current = Some(LastValueData {
                value: number.clone(),
                timestamp: SystemTime::now(),
            });
        })
    }
    fn checkpoint(&self, _descriptor: &Descriptor) {
        let _lock = self.inner.lock().map(|mut inner| {
            inner.checkpoint = inner.current.take();
        });
    }
    fn merge(
        &self,
        other: &std::sync::Arc<dyn Aggregator + Send + Sync>,
        _descriptor: &Descriptor,
    ) -> Result<()> {
        if let Some(other) = other.as_any().downcast_ref::<Self>() {
            self.inner.lock().map_err(From::from).and_then(|mut inner| {
                other.inner.lock().map_err(From::from).map(|mut other| {
                    match (&inner.checkpoint, &other.checkpoint) {
                        // Take if other timestamp is greater
                        (Some(checkpoint), Some(other_checkpoint))
                            if other_checkpoint.timestamp > checkpoint.timestamp =>
                        {
                            inner.checkpoint = other.checkpoint.take()
                        }
                        // Take if no value exists currently
                        (None, Some(_)) => inner.checkpoint = other.checkpoint.take(),
                        // Otherwise done
                        _ => (),
                    }
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

impl LastValue for LastValueAggregator {
    fn last_value(&self) -> Result<(Number, SystemTime)> {
        self.inner.lock().map_err(Into::into).and_then(|inner| {
            if let Some(checkpoint) = &inner.checkpoint {
                Ok((checkpoint.value.clone(), checkpoint.timestamp))
            } else {
                Err(MetricsError::NoDataCollected)
            }
        })
    }
}

/// TODO
#[derive(Debug, Default)]
struct Inner {
    current: Option<LastValueData>,
    checkpoint: Option<LastValueData>,
}

/// TODO
#[derive(Debug)]
struct LastValueData {
    value: Number,
    timestamp: SystemTime,
}
