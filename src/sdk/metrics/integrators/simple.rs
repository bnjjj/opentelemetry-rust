use crate::api::metrics;
use crate::sdk::export::metrics::{AggregationSelector, Integrator, Record};
use std::collections::HashMap;

/// TODO
#[derive(Debug)]
pub struct SimpleIntegrator {
    aggregation_selector: Box<dyn AggregationSelector>,
    stateful: bool,
    batch: HashMap<BatchKey, BatchValue>,
}

/// TODO
#[derive(Debug, PartialEq, Eq, Hash)]
struct BatchKey {}

/// TODO
#[derive(Debug)]
struct BatchValue {}

/// TODO
pub fn simple(selector: Box<dyn AggregationSelector>, stateful: bool) -> SimpleIntegrator {
    SimpleIntegrator {
        aggregation_selector: selector,
        stateful,
        batch: HashMap::default(),
    }
}

impl Integrator for SimpleIntegrator {
    fn process(&self, record: Record) -> metrics::Result<()> {
        todo!()
    }
}
