//! Metrics SDK API
use crate::api::metrics::{AsyncRunner, Descriptor, Measurement, Number, Result};
use crate::api::{Context, KeyValue};
use std::any::Any;
use std::fmt;
use std::sync::Arc;

/// TODO
pub trait MeterCore: fmt::Debug {
    /// TODO
    fn record_batch_with_context(
        &self,
        cx: &Context,
        labels: &[KeyValue],
        measurements: Vec<Measurement>,
    );

    /// TODO
    fn new_sync_instrument(&self, descriptor: Descriptor) -> Result<Arc<dyn SyncInstrument>>;

    /// TODO
    fn new_async_instrument(
        &self,
        descriptor: Descriptor,
        runner: AsyncRunner,
    ) -> Result<Arc<dyn AsyncInstrument>>;
}

/// TODO
pub trait Instrument: fmt::Debug {
    /// Description of the instrument's descriptor
    fn descriptor(&self) -> &str;
}

/// TODO
pub trait SyncInstrument: fmt::Debug {
    /// TODO
    fn bind<'a>(&self, labels: &'a [KeyValue]) -> Arc<dyn BoundSyncInstrument>;

    /// TODO
    fn record_one<'a>(&self, number: Number, labels: &'a [KeyValue]) {
        self.record_one_with_context(&Context::current(), number, labels)
    }

    /// TODO
    fn record_one_with_context<'a>(&self, cx: &Context, number: Number, labels: &'a [KeyValue]);

    /// Returns self as any
    fn as_any(&self) -> &dyn Any;
}

/// TODO
pub trait BoundSyncInstrument: fmt::Debug {
    /// TODO
    fn record_one<'a>(&self, number: Number) {
        self.record_one_with_context(&Context::current(), number)
    }

    /// TODO
    fn record_one_with_context<'a>(&self, cx: &Context, number: Number);
}

/// TODO
pub trait AsyncInstrument: Instrument {
    /// TODO
    fn as_any(&self) -> &dyn Any;
}
