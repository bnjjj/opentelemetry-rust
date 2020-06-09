use crate::api::{
    metrics::{Descriptor, Number, Result},
    Context,
};
use crate::sdk::export::metrics::Aggregator;
use std::any::Any;
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
        number: &Number,
        descriptor: &Descriptor,
    ) -> Result<()> {
        self.current.add(descriptor.number_kind(), number);
        Ok(())
    }
    fn checkpoint(&self, descriptor: &Descriptor) {
        todo!()
    }
    fn merge(
        &self,
        other: &Arc<dyn Aggregator + Send + Sync>,
        descriptor: &Descriptor,
    ) -> Result<()> {
        todo!()
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}
