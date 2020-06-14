//! Metrics SDK API
use crate::api::metrics::{AsyncRunner, Descriptor, Measurement, Number, Result};
use crate::api::{Context, KeyValue};
use std::any::Any;
use std::fmt;
use std::sync::Arc;

/// The interface an SDK must implement to supply a Meter implementation.
pub trait MeterCore: fmt::Debug {
    /// Atomically record a batch of measurements.
    fn record_batch_with_context(
        &self,
        cx: &Context,
        labels: &[KeyValue],
        measurements: Vec<Measurement>,
    );

    /// Create a new synchronous instrument implementation.
    fn new_sync_instrument(
        &self,
        descriptor: Descriptor,
    ) -> Result<Arc<dyn SyncInstrument + Send + Sync>>;

    /// Create a new asynchronous instrument implementation.
    fn new_async_instrument(
        &self,
        descriptor: Descriptor,
        runner: AsyncRunner,
    ) -> Result<Arc<dyn AsyncInstrument + Send + Sync>>;
}

/// A common interface for synchronous and asynchronous instruments.
pub trait Instrument: fmt::Debug {
    /// Description of the instrument's descriptor
    fn descriptor(&self) -> &Descriptor;
}

/// The implementation-level interface to a generic synchronous instrument
/// (e.g., ValueRecorder and Counter instruments).
pub trait SyncInstrument: Instrument {
    /// Creates an implementation-level bound instrument, binding a label set
    /// with this instrument implementation.
    fn bind<'a>(&self, labels: &'a [KeyValue]) -> Arc<dyn SyncBoundInstrument + Send + Sync>;

    /// Capture a single synchronous metric event.
    fn record_one<'a>(&self, number: Number, labels: &'a [KeyValue]) {
        self.record_one_with_context(&Context::current(), number, labels)
    }

    /// Capture a single synchronous metric event with context.
    fn record_one_with_context<'a>(&self, cx: &Context, number: Number, labels: &'a [KeyValue]);

    /// Returns self as any
    fn as_any(&self) -> &dyn Any;
}

/// The implementation-level interface to a generic synchronous bound instrument
pub trait SyncBoundInstrument: fmt::Debug + Send + Sync {
    /// Capture a single synchronous metric event.
    fn record_one(&self, number: Number) {
        self.record_one_with_context(&Context::current(), number)
    }

    /// Capture a single synchronous metric event with context.
    fn record_one_with_context(&self, cx: &Context, number: Number);
}

/// An implementation-level interface to an asynchronous instrument (e.g.,
/// Observer instruments).
pub trait AsyncInstrument: Instrument {
    /// The underlying type as `Any` to support downcasting.
    fn as_any(&self) -> &dyn Any;
}
