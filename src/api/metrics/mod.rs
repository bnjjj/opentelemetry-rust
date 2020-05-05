//! # OpenTelemetry Metrics API
//! The user-facing metrics API supports producing diagnostic measurements
//! using three basic kinds of instrument. "Metrics" are the thing being
//! produced--mathematical, statistical summaries of certain observable
//! behavior in the program. `Instrument`s are the devices used by the
//! program to record observations about their behavior. Therefore, we use
//! "metric instrument" to refer to a program object, allocated through the
//! API, used for recording metrics. There are three distinct instruments
//! in the Metrics API, commonly known as `Counter`s, `Gauge`s, and
//! `Measure`s.
//!
//! Monitoring and alerting are the common use-case for the data provided
//! through metric instruments, after various collection and aggregation
//! strategies are applied to the data. We find there are many other uses
//! for the metric events that stream into these instruments. We imagine
//! metric data being aggregated and recorded as events in tracing and
//! logging systems too, and for this reason OpenTelemetry requires a
//! separation of the API from the SDK.
//!
//! To capture measurements using an `Instrument`, you need an SDK that
//! implements the `Meter` API.
//!
//! ## Metric kinds and inputs
//!
//! The API distinguishes metric instruments by semantic meaning, not by
//! the type of value produced in an exporter.  This is a departure from
//! convention, compared with a number of common metric libraries, and
//! stems from the separation of the API and the SDK.  The SDK ultimately
//! determines how to handle metric events and could potentially implement
//! non-standard behavior.
//!
//! This explains why the metric API does not have metric instrument kinds
//! for exporting "Histogram" and "Summary" distribution explicitly, for
//! example.  These are both semantically `Measure` instruments and an SDK
//! can be configured to produce histograms or distribution summaries from
//! Measure events.  It is out of scope for the Metrics API to specify how
//! these alternatives are configured in a particular SDK.
//!
//! We believe the three metric kinds `Counter`, `Gauge`, and `Measure`
//! form a sufficient basis for expression of a wide variety of metric data.
//! Programmers write and read these as `add()`, `set()`, and `record()`
//! method calls, signifying the semantics and standard interpretation,
//! and we believe these three methods are all that are needed.
//!
//! Nevertheless, it is common to apply restrictions on metric values, the
//! inputs to `add()`, `set()`, and `record()`, in order to refine their
//! standard interpretation.  Generally, there is a question of whether
//! the instrument can be used to compute a rate, because that is usually
//! a desirable analysis.  Each metric instrument offers an optional
//! declaration, specifying restrictions on values input to the metric.
//! For example, Measures are declared as non-negative by default,
//! appropriate for reporting sizes and durations; a Measure option is
//! provided to record positive or negative values, but it does not change
//! the kind of instrument or the method name used, as the semantics are
//! unchanged.
use crate::api;
use std::sync::Arc;

pub mod counter;
pub mod measure;
pub mod meter;
pub mod noop;
pub mod observer;
pub mod value;

pub use meter::Meter;
use value::MeasurementValue;

/// The implementation-level interface to Set/Add/Record individual
/// metrics without precomputed labels.
pub trait Instrument<LS> {
    /// Allows the SDK to observe a single metric event for a given set of labels.
    fn record_one(&self, value: MeasurementValue, label_set: &LS);
}

/// The implementation-level interface to Set/Add/Record individual
/// metrics with precomputed labels.
pub trait InstrumentHandle {
    /// Allows the SDK to observe a single metric event.
    fn record_one(&self, value: MeasurementValue);
}

/// `LabelSet` is an implementation-level interface that represents a
/// set of `KeyValue` for use as pre-defined labels in the metrics API.
pub trait LabelSet {}

pub enum InstrumentKind {
    Measurement,
    Observer,
    Counter,
}

/// Contains some options for metrics of any kind.
pub struct Config {
    /// Description is an optional field describing the metric
    /// instrument.
    description: String,
    /// Unit is an optional field describing the metric instrument.
    unit: api::Unit,
    /// LibraryName is the name given to the Meter that created
    /// this instrument.  See `Provider`.
    library_name: String,
}

pub struct Descriptor {
    name: String,
    kind: InstrumentKind,
    config: Config,
}

/// `MetricOptions` contains some options for metrics of any kind.
#[derive(Default, Debug)]
pub struct MetricOptions {
    /// Description is an optional field describing the metric instrument.
    pub description: String,

    /// Unit is an optional field describing the metric instrument.
    /// Valid values are specified according to the
    /// [UCUM](http://unitsofmeasure.org/ucum.html).
    pub unit: api::Unit,

    /// Keys are dimension names for the given metric.
    pub keys: Vec<api::Key>,

    /// Alternate defines the property of metric value dependent on
    /// a metric type.
    ///
    /// - for `Counter`, `true` implies that the metric is an up-down
    ///   `Counter`
    ///
    /// - for `Gauge`, `true` implies that the metric is a
    ///   non-descending `Gauge`
    ///
    /// - for `Measure`, `true` implies that the metric supports
    ///   positive and negative values
    pub alternate: bool,
}

impl MetricOptions {
    /// Set a description for the current set of options.
    pub fn with_description<S: Into<String>>(self, description: S) -> Self {
        MetricOptions {
            description: description.into(),
            ..self
        }
    }

    /// Set a `Unit` for the current set of metric options.
    pub fn with_unit(self, unit: api::Unit) -> Self {
        MetricOptions { unit, ..self }
    }

    /// Set a list of `Key`s for the current set metric of options.
    pub fn with_keys(self, keys: Vec<api::Key>) -> Self {
        MetricOptions { keys, ..self }
    }

    /// Set monotonic for the given set of metric options.
    pub fn with_monotonic(self, _monotonic: bool) -> Self {
        // TODO figure out counter vs gauge issue here.
        unimplemented!()
    }

    /// Set absolute for the given set of metric options.
    pub fn with_absolute(self, absolute: bool) -> Self {
        MetricOptions {
            alternate: !absolute,
            ..self
        }
    }
}

/// Used to record `MeasurementValue`s for a given `Instrument` for use in
/// batch recording by a `Meter`.
#[allow(missing_debug_implementations)]
pub struct Measurement<LS> {
    instrument: Arc<dyn Instrument<LS>>,
    value: MeasurementValue,
}

impl<LS: LabelSet> Measurement<LS> {
    /// Create a new measurement
    pub fn new(instrument: Arc<dyn Instrument<LS>>, value: MeasurementValue) -> Self {
        Measurement { instrument, value }
    }

    /// Returns an instrument that created this measurement.
    pub fn instrument(&self) -> Arc<dyn Instrument<LS>> {
        self.instrument.clone()
    }

    /// Returns a value recorded in this measurement.
    pub fn into_value(self) -> MeasurementValue {
        self.value
    }
}

/// Metric Errors
#[derive(Debug)]
pub enum Error {
    /// TODO
    Tmp(String),
}
