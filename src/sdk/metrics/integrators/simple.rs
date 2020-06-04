use crate::api::metrics::{MetricsError, Result};
use crate::sdk::export::metrics::{AggregationSelector, Integrator, Record};
use std::collections::HashMap;
use std::sync::Mutex;

/// TODO
#[derive(Debug)]
pub struct SimpleIntegrator {
    aggregation_selector: Box<dyn AggregationSelector + Send + Sync>,
    stateful: bool,
    inner: Mutex<SimpleIntegratorInner>,
}

impl SimpleIntegrator {
    /// TODO
    pub fn try_lock_inner<F, T>(&self, mut f: F) -> Result<T>
    where
        F: FnMut(&mut SimpleIntegratorInner) -> T,
    {
        self.inner
            .try_lock()
            .map_err(|lock_err| MetricsError::Other(lock_err.to_string()))
            .map(|mut inner| f(&mut inner))
    }
}

#[derive(Debug)]
pub struct SimpleIntegratorInner {
    batch: HashMap<BatchKey, BatchValue>,
}

/// TODO
#[derive(Debug, PartialEq, Eq, Hash)]
struct BatchKey {}

/// TODO
#[derive(Debug)]
struct BatchValue {}

/// TODO
pub fn simple(
    selector: Box<dyn AggregationSelector + Send + Sync>,
    stateful: bool,
) -> SimpleIntegrator {
    SimpleIntegrator {
        aggregation_selector: selector,
        stateful,
        inner: Mutex::new(SimpleIntegratorInner {
            batch: HashMap::default(),
        }),
    }
}

impl Integrator for SimpleIntegrator {
    fn aggregation_selector(&self) -> &dyn AggregationSelector {
        self.aggregation_selector.as_ref()
    }
    fn process(&self, record: Record) -> Result<()> {
        todo!()
    }
}
