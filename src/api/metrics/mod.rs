//! # OpenTelemetry Metrics API
// use crate::api;
// use std::sync::Arc;
//
// pub mod counter;
// pub mod gauge;
// pub mod measure;
// pub mod noop;
// pub mod value;
//
// use counter::Counter;
// use gauge::Gauge;
// use measure::Measure;
// use value::MeasurementValue;
//
// /// The implementation-level interface to Set/Add/Record individual
// /// metrics without precomputed labels.
// pub trait Instrument<LS> {
//     /// Allows the SDK to observe a single metric event for a given set of labels.
//     fn record_one(&self, value: MeasurementValue, label_set: &LS);
// }
//
// /// The implementation-level interface to Set/Add/Record individual
// /// metrics with precomputed labels.
// pub trait InstrumentHandle {
//     /// Allows the SDK to observe a single metric event.
//     fn record_one(&self, value: MeasurementValue);
// }
//
// /// `LabelSet` is an implementation-level interface that represents a
// /// set of `KeyValue` for use as pre-defined labels in the metrics API.
// pub trait LabelSet {}
//
// /// `MetricOptions` contains some options for metrics of any kind.
// #[derive(Default, Debug)]
// pub struct MetricOptions {
//     /// Description is an optional field describing the metric instrument.
//     pub description: String,
//
//     /// Unit is an optional field describing the metric instrument.
//     /// Valid values are specified according to the
//     /// [UCUM](http://unitsofmeasure.org/ucum.html).
//     pub unit: api::Unit,
//
//     /// Keys are dimension names for the given metric.
//     pub keys: Vec<api::Key>,
//
//     /// Alternate defines the property of metric value dependent on
//     /// a metric type.
//     ///
//     /// - for `Counter`, `true` implies that the metric is an up-down
//     ///   `Counter`
//     ///
//     /// - for `Gauge`, `true` implies that the metric is a
//     ///   non-descending `Gauge`
//     ///
//     /// - for `Measure`, `true` implies that the metric supports
//     ///   positive and negative values
//     pub alternate: bool,
// }
//
// impl MetricOptions {
//     /// Set a description for the current set of options.
//     pub fn with_description<S: Into<String>>(self, description: S) -> Self {
//         MetricOptions {
//             description: description.into(),
//             ..self
//         }
//     }
//
//     /// Set a `Unit` for the current set of metric options.
//     pub fn with_unit(self, unit: api::Unit) -> Self {
//         MetricOptions { unit, ..self }
//     }
//
//     /// Set a list of `Key`s for the current set metric of options.
//     pub fn with_keys(self, keys: Vec<api::Key>) -> Self {
//         MetricOptions { keys, ..self }
//     }
//
//     /// Set monotonic for the given set of metric options.
//     pub fn with_monotonic(self, _monotonic: bool) -> Self {
//         // TODO figure out counter vs gauge issue here.
//         unimplemented!()
//     }
//
//     /// Set absolute for the given set of metric options.
//     pub fn with_absolute(self, absolute: bool) -> Self {
//         MetricOptions {
//             alternate: !absolute,
//             ..self
//         }
//     }
// }
//
// /// Used to record `MeasurementValue`s for a given `Instrument` for use in
// /// batch recording by a `Meter`.
// #[allow(missing_debug_implementations)]
// pub struct Measurement<LS> {
//     instrument: Arc<dyn Instrument<LS>>,
//     value: MeasurementValue,
// }
//
// impl<LS: LabelSet> Measurement<LS> {
//     /// Create a new measurement
//     pub fn new(instrument: Arc<dyn Instrument<LS>>, value: MeasurementValue) -> Self {
//         Measurement { instrument, value }
//     }
//
//     /// Returns an instrument that created this measurement.
//     pub fn instrument(&self) -> Arc<dyn Instrument<LS>> {
//         self.instrument.clone()
//     }
//
//     /// Returns a value recorded in this measurement.
//     pub fn into_value(self) -> MeasurementValue {
//         self.value
//     }
// }
//
// /// Meter is an interface to the metrics portion of the OpenTelemetry SDK.
// ///
// /// The Meter interface allows creating of a registered metric instrument using methods specific to
// /// each kind of metric. There are six constructors representing the three kinds of instrument
// /// taking either floating point or integer inputs, see the detailed design below.
// ///
// /// Binding instruments to a single Meter instance has two benefits:
// ///
// ///    1. Instruments can be exported from the zero state, prior to first use, with no explicit
// ///       Register call
// ///    2. The component name provided by the named Meter satisfies a namespace requirement
// ///
// /// The recommended practice is to define structures to contain the instruments in use and keep
// /// references only to the instruments that are specifically needed.
// ///
// /// We recognize that many existing metric systems support allocating metric instruments statically
// /// and providing the Meter interface at the time of use. In this example, typical of statsd
// /// clients, existing code may not be structured with a convenient place to store new metric
// /// instruments. Where this becomes a burden, it is recommended to use the global meter factory to
// /// construct a static named Meter, to construct metric instruments.
// ///
// /// The situation is similar for users of Prometheus clients, where instruments are allocated
// /// statically and there is an implicit global. Such code may not have access to the appropriate
// /// Meter where instruments are defined. Where this becomes a burden, it is recommended to use the
// /// global meter factory to construct a static named Meter, to construct metric instruments.
// ///
// /// Applications are expected to construct long-lived instruments. Instruments are considered
// /// permanent for the lifetime of a SDK, there is no method to delete them.
// pub trait Meter {
//     /// The `LabelSet` data type for this meter.
//     type LabelSet: LabelSet;
//     /// The `I64Counter` data type for this meter.
//     type I64Counter: Counter<i64, Self::LabelSet>;
//     /// The `F64Counter` data type for this meter.
//     type F64Counter: Counter<f64, Self::LabelSet>;
//     /// The `I64Gauge` data type for this meter.
//     type I64Gauge: Gauge<i64, Self::LabelSet>;
//     /// The `F64Gauge` data type for this meter.
//     type F64Gauge: Gauge<f64, Self::LabelSet>;
//     /// The `I64Measure` data type for this meter.
//     type I64Measure: Measure<i64, Self::LabelSet>;
//     /// The `F64Measure` data type for this meter.
//     type F64Measure: Measure<f64, Self::LabelSet>;
//
//     /// Returns a reference to a set of labels that cannot be read by the application.
//     fn labels(&self, key_values: Vec<api::KeyValue>) -> Self::LabelSet;
//
//     /// Creates a new `i64` counter with a given name and customized with passed options.
//     fn new_i64_counter<S: Into<String>>(&self, name: S, opts: MetricOptions) -> Self::I64Counter;
//
//     /// Creates a new `f64` counter with a given name and customized with passed options.
//     fn new_f64_counter<S: Into<String>>(&self, name: S, opts: MetricOptions) -> Self::F64Counter;
//
//     /// Creates a new `i64` gauge with a given name and customized with passed options.
//     fn new_i64_gauge<S: Into<String>>(&self, name: S, opts: MetricOptions) -> Self::I64Gauge;
//
//     /// Creates a new `f64` gauge with a given name and customized with passed options.
//     fn new_f64_gauge<S: Into<String>>(&self, name: S, opts: MetricOptions) -> Self::F64Gauge;
//
//     /// Creates a new `i64` measure with a given name and customized with passed options.
//     fn new_i64_measure<S: Into<String>>(&self, name: S, opts: MetricOptions) -> Self::I64Measure;
//
//     /// Creates a new `f64` measure with a given name and customized with passed options.
//     fn new_f64_measure<S: Into<String>>(&self, name: S, opts: MetricOptions) -> Self::F64Measure;
//
//     /// Atomically records a batch of measurements.
//     fn record_batch<M: IntoIterator<Item = Measurement<Self::LabelSet>>>(
//         &self,
//         label_set: &Self::LabelSet,
//         measurements: M,
//     );
// }
//

use crate::api::{Context, KeyValue};
use std::fmt;
use std::marker;
use std::result;
use std::sync::Arc;
use thiserror::Error;

mod config;
mod descriptor;
pub mod noop;
mod number;
pub mod registry;
pub mod sdk_api;
mod sync;
mod value_recorder;

pub use config::Config;
pub use descriptor::Descriptor;
pub use number::Number;
pub use sync::{F64ObserverResult, Measurement};
pub use value_recorder::{ValueRecorder, ValueRecorderBuilder};

/// TODO
pub type Result<T> = result::Result<T, MetricsError>;

/// TODO
#[derive(Error, Debug)]
pub enum MetricsError {
    /// TODO
    #[error("unknown metrics error")]
    Unknown,
    /// TODO
    #[error("the requested quantile is out of range")]
    InvalidQuantile,
}

/// TODO
#[derive(Debug)]
pub struct Meter {
    name: String,
    core: Arc<dyn sdk_api::MeterCore>,
}

impl Meter {
    /// TODO
    pub fn new<T: Into<String>>(name: T, core: Arc<dyn sdk_api::MeterCore>) -> Self {
        Meter {
            name: name.into(),
            core,
        }
    }

    /// TODO
    pub fn f64_value_observer<T, F>(&self, name: T, callback: F) -> ValueObserverBuilder<f64>
    where
        Self: Sized,
        T: Into<String>,
        F: Fn(F64ObserverResult) + 'static,
    {
        ValueObserverBuilder {
            meter: self,
            name: name.into(),
            runner: Runner::F64Async(Box::new(callback)),
            description: None,
            _marker: std::marker::PhantomData,
        }
    }

    /// TODO
    pub fn f64_value_recorder<T>(&self, name: T) -> ValueRecorderBuilder<f64>
    where
        Self: Sized,
        T: Into<String>,
    {
        ValueRecorderBuilder {
            meter: self,
            descriptor: Descriptor::new(
                name.into(),
                self.name.clone(),
                InstrumentKind::ValueRecorder,
                NumberKind::F64,
            ),
            _marker: marker::PhantomData,
        }
    }

    /// TODO
    pub fn new_sync_instrument(
        &self,
        descriptor: Descriptor,
    ) -> Result<Arc<dyn sdk_api::SyncInstrument>> {
        self.core.new_sync_instrument(descriptor)
    }

    /// TODO
    pub fn record_batch<T: IntoIterator<Item = Measurement>>(
        &self,
        labels: &[KeyValue],
        measurements: T,
    ) {
        self.record_batch_with_context(&Context::current(), labels, measurements)
    }

    /// TODO
    pub fn record_batch_with_context<T: IntoIterator<Item = Measurement>>(
        &self,
        cx: &Context,
        labels: &[KeyValue],
        measurements: T,
    ) {
        self.core
            .record_batch_with_context(cx, labels, measurements.into_iter().collect())
    }
}

/// TODO
pub trait MeterProvider: fmt::Debug {
    /// TODO
    fn meter(&self, name: &str) -> Meter;
}

/// TODO
#[derive(Debug)]
pub enum InstrumentKind {
    /// TODO
    ValueObserver,
    /// TODO
    ValueRecorder,
}

/// TODO
#[derive(Debug)]
pub enum NumberKind {
    /// TODO
    F64,
}

/// TODO
#[derive(Debug)]
pub struct AsyncInstrument;

/// TODO
#[derive(Debug)]
pub struct ValueObserver<T> {
    _marker: std::marker::PhantomData<T>,
}

/// TODO
#[derive(Debug)]
pub struct ValueObserverBuilder<'a, T> {
    meter: &'a Meter,
    name: String,
    runner: Runner,
    description: Option<String>,
    _marker: std::marker::PhantomData<T>,
}

impl<T> ValueObserverBuilder<'_, T> {
    /// TODO
    pub fn with_description<S: Into<String>>(self, description: S) -> Self {
        ValueObserverBuilder {
            description: Some(description.into()),
            ..self
        }
    }

    /// TODO
    pub fn init(self) -> ValueObserver<T> {
        ValueObserver {
            _marker: std::marker::PhantomData,
        }
    }
}

/// TODO
pub enum Runner {
    /// TODO
    F64Async(Box<dyn Fn(F64ObserverResult)>),
}
impl fmt::Debug for Runner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Runner::F64Async(_) => f
                .debug_struct("Runner")
                .field("closure", &"Fn(F64ObserverResult)")
                .finish(),
        }
    }
}

// /// TODO
// pub trait Counter: Instrument {
//     /// TODO
//     fn add(&self, increment: usize);
// }

// /// TODO
// pub trait UpDownCounter: Instrument {
//     /// TODO
//     fn add(&self, increment: isize);
// }

// /// TODO
// pub trait ValueRecorder: Instrument {
//     /// TODO
//     fn record(&self, increment: isize);
// }

// /// TODO
// pub trait SumObserver: Instrument {
//     /// TODO
//     fn observe(&self, increment: usize);
// }
//
// /// TODO
// pub trait UpDownSumObserver: Instrument {
//     /// TODO
//     fn observe(&self, increment: isize);
// }
//
// TODO
// pub trait ValueObserver: Instrument {
//     /// TODO
//     fn observe(&self, increment: isize);
// }
