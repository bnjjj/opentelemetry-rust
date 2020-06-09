//! Simple Metric Selectors
use crate::api::metrics::{Descriptor, InstrumentKind};
use crate::sdk::export::metrics::{AggregationSelector, Aggregator};
use crate::sdk::metrics::aggregators;
use std::sync::Arc;

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
    fn aggregator_for(&self, descriptor: &Descriptor) -> Option<Arc<dyn Aggregator + Send + Sync>> {
        match descriptor.instrument_kind() {
            InstrumentKind::ValueObserver | InstrumentKind::ValueRecorder => {
                Some(aggregators::min_max_sum_count(descriptor))
            }
            _ => Some(aggregators::sum()),
        }
    }
}
