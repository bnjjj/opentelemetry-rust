//! # OpenTelemetry Meter API
use super::Error;
use crate::api::{self, Counter, LabelSet, Measure, Measurement, MetricOptions, Observer};

/// Meter is an interface to the metrics portion of the OpenTelemetry SDK.
///
/// The Meter interface allows creating of a registered metric instrument using methods specific to
/// each kind of metric. There are six constructors representing the three kinds of instrument
/// taking either floating point or integer inputs, see the detailed design below.
///
/// Binding instruments to a single Meter instance has two benefits:
///
///    1. Instruments can be exported from the zero state, prior to first use, with no explicit
///       Register call
///    2. The component name provided by the named Meter satisfies a namespace requirement
///
/// The recommended practice is to define structures to contain the instruments in use and keep
/// references only to the instruments that are specifically needed.
///
/// We recognize that many existing metric systems support allocating metric instruments statically
/// and providing the Meter interface at the time of use. In this example, typical of statsd
/// clients, existing code may not be structured with a convenient place to store new metric
/// instruments. Where this becomes a burden, it is recommended to use the global meter factory to
/// construct a static named Meter, to construct metric instruments.
///
/// The situation is similar for users of Prometheus clients, where instruments are allocated
/// statically and there is an implicit global. Such code may not have access to the appropriate
/// Meter where instruments are defined. Where this becomes a burden, it is recommended to use the
/// global meter factory to construct a static named Meter, to construct metric instruments.
///
/// Applications are expected to construct long-lived instruments. Instruments are considered
/// permanent for the lifetime of a SDK, there is no method to delete them.
pub trait Meter {
    /// The `LabelSet` data type for this meter.
    type LabelSet: LabelSet;
    /// The `I64Counter` data type for this meter.
    type I64Counter: Counter<i64, Self::LabelSet>;
    /// The `F64Counter` data type for this meter.
    type F64Counter: Counter<f64, Self::LabelSet>;
    /// The `I64Observer` data type for this meter.
    type I64Observer: Observer<i64, Self::LabelSet>;
    /// The `F64Observer` data type for this meter.
    type F64Observer: Observer<f64, Self::LabelSet>;
    /// The `I64Measure` data type for this meter.
    type I64Measure: Measure<i64, Self::LabelSet>;
    /// The `F64Measure` data type for this meter.
    type F64Measure: Measure<f64, Self::LabelSet>;

    /// Returns a reference to a set of labels that cannot be read by the application.
    fn labels(&self, key_values: Vec<api::KeyValue>) -> Self::LabelSet;

    /// Creates a new `i64` counter with a given name and customized with passed options.
    fn new_i64_counter<S: Into<String>>(
        &self,
        name: S,
        opts: MetricOptions,
    ) -> Result<Self::I64Counter, Error>;

    /// Creates a new `f64` counter with a given name and customized with passed options.
    fn new_f64_counter<S: Into<String>>(
        &self,
        name: S,
        opts: MetricOptions,
    ) -> Result<Self::F64Counter, Error>;

    /// Creates a new `i64` observer with a given name and customized with passed options.
    fn new_i64_observer<S: Into<String>>(
        &self,
        name: S,
        opts: MetricOptions,
    ) -> Result<Self::I64Observer, Error>;

    /// Creates a new `f64` observer with a given name and customized with passed options.
    fn new_f64_observer<S: Into<String>>(
        &self,
        name: S,
        opts: MetricOptions,
    ) -> Result<Self::F64Observer, Error>;

    /// Creates a new `i64` measure with a given name and customized with passed options.
    fn new_i64_measure<S: Into<String>>(
        &self,
        name: S,
        opts: MetricOptions,
    ) -> Result<Self::I64Measure, Error>;

    /// Creates a new `f64` measure with a given name and customized with passed options.
    fn new_f64_measure<S: Into<String>>(
        &self,
        name: S,
        opts: MetricOptions,
    ) -> Result<Self::F64Measure, Error>;

    /// Atomically records a batch of measurements.
    fn record_batch<M: IntoIterator<Item = Measurement<Self::LabelSet>>>(
        &self,
        label_set: &Self::LabelSet,
        measurements: M,
    );
}
