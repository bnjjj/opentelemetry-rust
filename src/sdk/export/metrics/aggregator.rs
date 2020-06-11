//! Metrics SDK Aggregator export API
use crate::api::metrics::{self, Descriptor, InstrumentKind, MetricsError, Number, NumberKind};
use crate::api::Context;
use std::any::Any;
use std::fmt;
use std::sync::Arc;

/// TODO
pub fn range_test(number: &Number, descriptor: &Descriptor) -> metrics::Result<()> {
    if descriptor.number_kind() == &NumberKind::F64 && number.is_nan() {
        return Err(MetricsError::NaNInput);
    }

    match descriptor.instrument_kind() {
        InstrumentKind::Counter | InstrumentKind::SumObserver => {
            if number.is_negative(descriptor.number_kind()) {
                return Err(MetricsError::NegativeInput);
            }
        }
        _ => (),
    };
    Ok(())
}

/// TODO
pub trait Aggregator: fmt::Debug {
    /// Update receives a new measured value and incorporates it
    /// into the aggregation.  Update() calls may arrive
    /// concurrently as the SDK does not provide synchronization.
    ///
    /// Descriptor.NumberKind() should be consulted to determine
    /// whether the provided number is an int64 or float64.
    ///
    /// The Context argument comes from user-level code and could be
    /// inspected for distributed or span context.
    fn update(&self, number: &Number, descriptor: &Descriptor) -> metrics::Result<()> {
        self.update_with_context(&Context::current(), number, descriptor)
    }

    /// TODO
    fn update_with_context(
        &self,
        cx: &Context,
        number: &Number,
        descriptor: &Descriptor,
    ) -> metrics::Result<()>;

    /// Checkpoint is called during collection to finish one period
    /// of aggregation by atomically saving the current value.
    /// Checkpoint() is called concurrently with Update().
    /// Checkpoint should reset the current state to the empty
    /// state, in order to begin computing a new delta for the next
    /// collection period.
    ///
    /// After the checkpoint is taken, the current value may be
    /// accessed using by converting to one a suitable interface
    /// types in the `aggregator` sub-package.
    ///
    /// The Context argument originates from the controller that
    /// orchestrates collection.
    fn checkpoint(&self, descriptor: &Descriptor);

    /// Merge combines the checkpointed state from the argument
    /// aggregator into this aggregator's checkpointed state.
    /// Merge() is called in a single-threaded context, no locking
    /// is required.
    fn merge(
        &self,
        other: &Arc<dyn Aggregator + Send + Sync>,
        descriptor: &Descriptor,
    ) -> metrics::Result<()>;

    /// TODO
    fn as_any(&self) -> &dyn Any;
}
