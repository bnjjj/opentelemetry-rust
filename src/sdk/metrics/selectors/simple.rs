//! Simple Metric Selectors
use crate::api::metrics;
use crate::sdk::export::metrics::{AggregationSelector, Aggregator};

/// TODO
#[derive(Debug)]
pub enum Selector {
    /// TODO
    Inexpensive,
    /// TODO
    Exact,
    /// TODO
    Sketch,
    /// TODO
    Histogram,
}

impl AggregationSelector for Selector {
    fn aggregator_for(&self, _descriptor: &metrics::Descriptor) -> &dyn Aggregator {
        todo!()
    }
}
