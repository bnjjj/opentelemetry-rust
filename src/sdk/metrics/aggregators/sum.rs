use crate::api::{metrics::Number, Context};
use crate::sdk::export::metrics::Aggregator;
use std::sync::Arc;

/// TODO
pub fn sum() -> Arc<dyn Aggregator + Send + Sync> {
    Arc::new(SumAggregator::default())
}

/// TODO
#[derive(Debug, Default)]
pub struct SumAggregator {
    current: Number,
    checkpoint: Number,
}

impl Aggregator for SumAggregator {
    fn update_with_context(
        &self,
        cx: &Context,
        number: Number,
        descriptor: &crate::api::metrics::Descriptor,
    ) -> Result<(), crate::api::metrics::MetricsError> {
        self.current.add(&descriptor.number_kind, number);
        Ok(())
    }
    fn checkpoint(&self, descriptor: &crate::api::metrics::Descriptor) {
        todo!()
    }
    fn merge(
        self,
        other: Box<dyn Aggregator>,
        descriptor: crate::api::metrics::Descriptor,
    ) -> Result<(), crate::api::metrics::MetricsError> {
        todo!()
    }
}
