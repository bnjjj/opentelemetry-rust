//! Metrics Export
use crate::api::{
    labels,
    metrics::{Descriptor, Result},
};
use crate::sdk::resource::Resource;
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
}

///TODO
pub trait LockedIntegrator {
    /// Process is called by the SDK once per internal record,
    /// passing the export Record (a Descriptor, the corresponding
    /// Labels, and the checkpointed Aggregator).
    ///
    /// The Context argument originates from the controller that
    /// orchestrates collection.
    fn process(&mut self, record: Record) -> Result<()>;

    /// TODO
    fn checkpoint_set(&mut self) -> &mut dyn CheckpointSet;

    /// TODO
    fn finished_collection(&mut self);
}

/// TODO
pub trait AggregationSelector: fmt::Debug {
    /// TODO
    fn aggregator_for(&self, descriptor: &Descriptor) -> Option<Arc<dyn Aggregator + Send + Sync>>;
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
    fn export(&self, checkpoint_set: &mut dyn CheckpointSet) -> Result<()>;
}

/// TODO
pub trait CheckpointSet: fmt::Debug {
    /// TODO
    fn try_for_each(&mut self, f: &mut dyn FnMut(&Record) -> Result<()>) -> Result<()>;
}

/// TODO
pub fn record<'a>(
    descriptor: &'a Descriptor,
    labels: &'a labels::Set,
    resource: &'a Arc<Resource>,
    aggregator: &'a Arc<dyn Aggregator + Send + Sync>,
) -> Record<'a> {
    Record {
        descriptor,
        labels,
        resource,
        aggregator,
    }
}

/// TODO
#[derive(Debug)]
pub struct Record<'a> {
    descriptor: &'a Descriptor,
    labels: &'a labels::Set,
    resource: &'a Resource,
    aggregator: &'a Arc<dyn Aggregator + Send + Sync>,
}

impl<'a> Record<'a> {
    /// TODO
    pub fn new(
        descriptor: &'a Descriptor,
        labels: &'a labels::Set,
        resource: &'a Resource,
        aggregator: &'a Arc<dyn Aggregator + Send + Sync>,
    ) -> Self {
        Record {
            descriptor,
            labels,
            resource,
            aggregator,
        }
    }
    /// TODO
    pub fn descriptor(&self) -> &Descriptor {
        self.descriptor
    }

    /// TODO
    pub fn labels(&self) -> &labels::Set {
        self.labels
    }

    /// TODO
    pub fn resource(&self) -> &Resource {
        self.resource
    }

    /// TODO
    pub fn aggregator(&self) -> &Arc<dyn Aggregator + Send + Sync> {
        self.aggregator
    }
}
