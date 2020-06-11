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
    Histogram(Vec<f64>),
}

impl AggregationSelector for Selector {
    fn aggregator_for(&self, descriptor: &Descriptor) -> Option<Arc<dyn Aggregator + Send + Sync>> {
        match self {
            Selector::Inexpensive => match descriptor.instrument_kind() {
                InstrumentKind::ValueObserver | InstrumentKind::ValueRecorder => {
                    Some(Arc::new(aggregators::min_max_sum_count(descriptor)))
                }
                _ => Some(Arc::new(aggregators::sum())),
            },
            Selector::Exact => match descriptor.instrument_kind() {
                InstrumentKind::ValueObserver | InstrumentKind::ValueRecorder => {
                    Some(Arc::new(aggregators::array()))
                }
                _ => Some(Arc::new(aggregators::sum())),
            },
            Selector::Histogram(boundaries) => match descriptor.instrument_kind() {
                InstrumentKind::ValueObserver | InstrumentKind::ValueRecorder => {
                    Some(Arc::new(aggregators::histogram(descriptor, &boundaries)))
                }
                _ => Some(Arc::new(aggregators::sum())),
            },
        }
    }
}
