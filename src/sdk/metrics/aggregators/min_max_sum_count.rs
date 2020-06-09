use crate::api::{
    metrics::{Descriptor, MetricsError, Number, NumberKind, Result},
    Context,
};
use crate::sdk::export::metrics::Aggregator;
use std::any::Any;
use std::cmp::Ordering;
use std::mem;
use std::sync::{Arc, Mutex};

/// TODO
pub fn min_max_sum_count(descriptor: &Descriptor) -> Arc<dyn Aggregator + Send + Sync> {
    let kind = descriptor.number_kind().clone();
    Arc::new(MinMaxSumCountAggregator {
        inner: Mutex::new(Inner {
            current: State::empty(&kind),
            checkpoint: None,
        }),
        kind,
    })
}

#[derive(Debug)]
struct Inner {
    current: State,
    checkpoint: Option<State>,
}

///TODO
#[derive(Debug)]
pub struct MinMaxSumCountAggregator {
    inner: Mutex<Inner>,
    kind: NumberKind,
}

impl MinMaxSumCountAggregator {
    /// TODO
    pub fn count(&self) -> Result<u64> {
        self.inner.try_lock().map_err(From::from).map(|inner| {
            inner
                .checkpoint
                .as_ref()
                .map_or(0, |state| state.count.to_u64())
        })
    }

    /// TODO
    pub fn max(&self) -> Result<Number> {
        self.inner.try_lock().map_err(From::from).map(|inner| {
            inner
                .checkpoint
                .as_ref()
                .map_or(0u64.into(), |state| state.max.clone())
        })
    }

    /// TODO
    pub fn min(&self) -> Result<Number> {
        self.inner.try_lock().map_err(From::from).map(|inner| {
            inner
                .checkpoint
                .as_ref()
                .map_or(0u64.into(), |state| state.min.clone())
        })
    }
}

impl Aggregator for MinMaxSumCountAggregator {
    fn update_with_context(
        &self,
        _cx: &Context,
        number: &Number,
        descriptor: &Descriptor,
    ) -> Result<()> {
        self.inner
            .try_lock()
            .map(|mut inner| {
                let current = &mut inner.current;
                let kind = descriptor.number_kind();
                // println!("UPDATING IN AGGREGATOR: {:?}", descriptor);

                current.count.add(&NumberKind::U64, &1u64.into());
                current.sum.add(kind, number);
                if number.partial_cmp(kind, &current.min) == Some(Ordering::Less) {
                    // println!(
                    //     "NEW MIN: {:?}, {:?}",
                    //     current.min.to_debug(kind),
                    //     number.to_debug(kind)
                    // );
                    current.min = number.clone();
                }
                if number.partial_cmp(kind, &current.max) == Some(Ordering::Greater) {
                    // println!(
                    //     "NEW MAX: {:?}, {:?}",
                    //     current.max.to_debug(kind),
                    //     number.to_debug(kind)
                    // );
                    current.max = number.clone();
                }
            })
            .map_err(From::from)
    }

    fn checkpoint(&self, _descriptor: &Descriptor) {
        let _ = self.inner.try_lock().map(|mut inner| {
            inner.checkpoint = Some(mem::replace(&mut inner.current, State::empty(&self.kind)))
        });
    }

    fn merge(
        &self,
        aggregator: &Arc<dyn Aggregator + Send + Sync>,
        descriptor: &Descriptor,
    ) -> Result<()> {
        // println!("MERGING, see strategy below");
        if let Some(other) = aggregator.as_any().downcast_ref::<Self>() {
            self.inner.lock().map_err(From::from).and_then(|mut inner| {
                other.inner.lock().map_err(From::from).and_then(|oi| {
                    match (inner.checkpoint.as_ref(), oi.checkpoint.as_ref()) {
                        (None, Some(other_checkpoint)) => {
                            // println!("Choosing right, no current");
                            // dbg!(other_checkpoint.min.to_debug(descriptor.number_kind()));
                            // dbg!(other_checkpoint.max.to_debug(descriptor.number_kind()));
                            inner.checkpoint = Some(other_checkpoint.clone());
                        }
                        (Some(_), None) | (None, None) => (),
                        (Some(cp), Some(ocp)) => {
                            // println!("MERGING IN AGGREGATOR: {:?}", descriptor);
                            cp.count.add(&NumberKind::U64, &ocp.count);
                            cp.sum.add(descriptor.number_kind(), &ocp.sum);

                            if cp.min.partial_cmp(descriptor.number_kind(), &ocp.min)
                                == Some(Ordering::Greater)
                            {
                                // dbg!(
                                //     "NEW MIN",
                                //     cp.min.to_debug(descriptor.number_kind()),
                                //     ocp.min.to_debug(descriptor.number_kind())
                                // );
                                cp.min.assign(descriptor.number_kind(), &ocp.min);
                            } else {
                                // dbg!(
                                //     "KEEPING MIN",
                                //     cp.min.to_debug(descriptor.number_kind()),
                                //     ocp.min.to_debug(descriptor.number_kind())
                                // );
                            }
                            if cp.max.partial_cmp(descriptor.number_kind(), &ocp.max)
                                == Some(Ordering::Less)
                            {
                                // dbg!(
                                //     "NEW MAX",
                                //     cp.max.to_debug(descriptor.number_kind()),
                                //     ocp.max.to_debug(descriptor.number_kind())
                                // );
                                cp.max.assign(descriptor.number_kind(), &ocp.max);
                            }
                        }
                    }
                    Ok(())
                })
            })
        } else {
            dbg!(Err(MetricsError::InconsistentMergeError(format!(
                "Expected {:?}, got: {:?}",
                self, aggregator
            ))))
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// TODO
#[derive(Debug)]
struct State {
    count: Number,
    sum: Number,
    min: Number,
    max: Number,
}

impl State {
    fn empty(kind: &NumberKind) -> Self {
        State {
            count: Number::default(),
            sum: kind.zero(),
            min: kind.max(),
            max: kind.min(),
        }
    }

    fn clone(&self) -> Self {
        State {
            count: self.count.clone(),
            sum: self.sum.clone(),
            min: self.min.clone(),
            max: self.max.clone(),
        }
    }
}
