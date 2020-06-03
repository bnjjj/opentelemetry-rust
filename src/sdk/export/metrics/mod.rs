//! Metrics Export
use crate::api::{metrics, metrics::MetricsError};
use std::fmt;
use std::sync::Arc;

pub mod aggregator;

pub use aggregator::Aggregator;

/// Integrator is responsible for deciding which kind of aggregation to
/// use (via AggregationSelector), gathering exported results from the
/// SDK during collection, and deciding over which dimensions to group
/// the exported data.
///
/// The SDK supports binding only one of these interfaces, as it has
/// the sole responsibility of determining which Aggregator to use for
/// each record.
///
/// The embedded AggregationSelector interface is called (concurrently)
/// in instrumentation context to select the appropriate Aggregator for
/// an instrument.
///
/// The `Process` method is called during collection in a
/// single-threaded context from the SDK, after the aggregator is
/// checkpointed, allowing the integrator to build the set of metrics
/// currently being exported.
pub trait Integrator: fmt::Debug {
    /// AggregationSelector is responsible for selecting the
    /// concrete type of Aggregator used for a metric in the SDK.
    ///
    /// This may be a static decision based on fields of the
    /// Descriptor, or it could use an external configuration
    /// source to customize the treatment of each metric
    /// instrument.
    ///
    /// The result from AggregatorSelector.AggregatorFor should be
    /// the same type for a given Descriptor or else nil.  The same
    /// type should be returned for a given descriptor, because
    /// Aggregators only know how to Merge with their own type.  If
    /// the result is nil, the metric instrument will be disabled.
    ///
    /// Note that the SDK only calls AggregatorFor when new records
    /// require an Aggregator. This does not provide a way to
    /// disable metrics with active records.
    fn aggregation_selector(&self) -> &dyn AggregationSelector;

    /// Process is called by the SDK once per internal record,
    /// passing the export Record (a Descriptor, the corresponding
    /// Labels, and the checkpointed Aggregator).
    ///
    /// The Context argument originates from the controller that
    /// orchestrates collection.
    fn process(&self, record: Record) -> metrics::Result<()>;
}

/// TODO
pub trait AggregationSelector: fmt::Debug {
    /// TODO
    fn aggregator_for(&self, descriptor: &metrics::Descriptor)
        -> Arc<dyn Aggregator + Send + Sync>;
}

/// TODO
pub trait Exporter: fmt::Debug {
    /// Export is called immediately after completing a collection
    /// pass in the SDK.
    ///
    /// The Context comes from the controller that initiated
    /// collection.
    ///
    /// The CheckpointSet interface refers to the Integrator that just
    /// completed collection.
    fn export(&self, checkpoint_set: &dyn CheckpointSet) -> Result<(), MetricsError>;
}

/// TODO
pub trait CheckpointSet {
    // // ForEach iterates over aggregated checkpoints for all
    // // metrics that were updated during the last collection
    // // period. Each aggregated checkpoint returned by the
    // // function parameter may return an error.
    // // ForEach tolerates ErrNoData silently, as this is
    // // expected from the Meter implementation. Any other kind
    // // of error will immediately halt ForEach and return
    // // the error to the caller.
    // ForEach(func(Record) error) error
    //
    // // Locker supports locking the checkpoint set.  Collection
    // // into the checkpoint set cannot take place (in case of a
    // // stateful integrator) while it is locked.
    // //
    // // The Integrator attached to the Accumulator MUST be called
    // // with the lock held.
    // sync.Locker
    //
    // // RLock acquires a read lock corresponding to this Locker.
    // RLock()
    // // RUnlock releases a read lock corresponding to this Locker.
    // RUnlock()
}

/// TODO
#[derive(Debug)]
pub struct Record {
    // descriptor *metric.Descriptor
// labels     *label.Set
// resource   *resource.Resource
// aggregator Aggregator
}
