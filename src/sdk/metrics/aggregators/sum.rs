use crate::api::{
    metrics::{Descriptor, Number, Result},
    Context,
};
use crate::sdk::{export::metrics::Aggregator, metrics::aggregators::Sum};
use std::any::Any;
use std::sync::Arc;

/// TODO
pub fn sum() -> SumAggregator {
    SumAggregator::default()
}

/// TODO
#[derive(Debug, Default)]
pub struct SumAggregator {
    current: Number,
    checkpoint: Number,
}

impl Sum for SumAggregator {
    fn sum(&self) -> Result<Number> {
        Ok(self.checkpoint.clone())
    }
}

impl Aggregator for SumAggregator {
    fn update_with_context(
        &self,
        _cx: &Context,
        number: &Number,
        descriptor: &Descriptor,
    ) -> Result<()> {
        self.current.add(descriptor.number_kind(), number);
        Ok(())
    }
    fn checkpoint(&self, descriptor: &Descriptor) {
        let kind = descriptor.number_kind();
        self.checkpoint.assign(kind, &self.current);
        self.current.assign(kind, &kind.zero());
    }
    fn merge(
        &self,
        other: &Arc<dyn Aggregator + Send + Sync>,
        descriptor: &Descriptor,
    ) -> Result<()> {
        if let Some(other_sum) = other.as_any().downcast_ref::<SumAggregator>() {
            self.checkpoint
                .add(descriptor.number_kind(), &other_sum.checkpoint)
        }

        Ok(())
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}
