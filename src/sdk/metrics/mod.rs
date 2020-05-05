//! # OpenTelemetry Metrics SDK
//!
//! The metrics SDK supports producing diagnostic measurements
//! using three basic kinds of `Instrument`s. "Metrics" are the thing being
//! produced--mathematical, statistical summaries of certain observable
//! behavior in the program. `Instrument`s are the devices used by the
//! program to record observations about their behavior. Therefore, we use
//! "metric instrument" to refer to a program object, allocated through the
//! `Meter` struct, used for recording metrics. There are three distinct
//! instruments in the Metrics API, commonly known as `Counter`s, `Gauge`s,
//! and `Measure`s.
//use crate::exporter::metrics::prometheus;
use crate::{api, api::metrics::Error, sdk::Exporter};
use std::borrow::Cow;
use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex};

/// Collection of label key and value types.
pub type LabelSet = HashMap<Cow<'static, str>, Cow<'static, str>>;
impl api::LabelSet for LabelSet {}

struct UniqueMetric {
    descriptor: api::metrics::Descriptor,
    ordered: BTreeMap<api::Key, api::Value>,
}

/// `Meter` implementation to create manage metric instruments and record
/// batch measurements
#[allow(missing_debug_implementations)]
pub struct Meter {
    current: Arc<Mutex<HashMap<UniqueMetric, Record>>>,
    async_instruments: Arc<Mutex<HashMap<Box<dyn std::any::Any>, Box<dyn std::any::Any>>>>,
    current_epoc: u64,
    exporter: Arc<Mutex<dyn Exporter>>,
}

pub struct Instrument {}
pub struct SyncInstrument {}
pub struct AsyncInstrument {}

// impl Meter {
//     /// Create a new `Meter` instance with a component name and empty registry.
//     pub fn new(component: &'static str) -> Self {
//         Meter {
//             registry: prometheus::default_registry(),
//             component,
//         }
//     }
//
//     /// Build prometheus `Opts` from `name` and `description`.
//     fn build_opts(
//         &self,
//         mut name: String,
//         unit: api::Unit,
//         description: String,
//     ) -> prometheus::Opts {
//         if !unit.as_str().is_empty() {
//             name.push_str(&format!("_{}", unit.as_str()));
//         }
//         // Prometheus cannot have empty help strings
//         let help = if !description.is_empty() {
//             description
//         } else {
//             format!("{} metric", name)
//         };
//         prometheus::Opts::new(name, help).namespace(self.component)
//     }
// }
//
// impl api::Meter for Meter {
//     /// The label set used by this `Meter`.
//     type LabelSet = LabelSet;
//     /// This implementation of `api::Meter` produces `prometheus::IntCounterVec;` instances.
//     type I64Counter = prometheus::IntCounterVec;
//     /// This implementation of `api::Meter` produces `prometheus::CounterVec;` instances.
//     type F64Counter = prometheus::CounterVec;
//     /// This implementation of `api::Meter` produces `prometheus::IntGaugeVec;` instances.
//     type I64Observer = prometheus::IntGaugeVec;
//     /// This implementation of `api::Meter` produces `prometheus::GaugeVec;` instances.
//     type F64Observer = prometheus::GaugeVec;
//     /// This implementation of `api::Meter` produces `prometheus::IntMeasure;` instances.
//     type I64Measure = prometheus::IntMeasure;
//     /// This implementation of `api::Meter` produces `prometheus::HistogramVec;` instances.
//     type F64Measure = prometheus::HistogramVec;
//
//     /// Builds a `LabelSet` from `KeyValue`s.
//     fn labels(&self, key_values: Vec<api::KeyValue>) -> Self::LabelSet {
//         let mut label_set: Self::LabelSet = Default::default();
//
//         for api::KeyValue { key, value } in key_values.into_iter() {
//             label_set.insert(Cow::Owned(key.into()), Cow::Owned(value.into()));
//         }
//
//         label_set
//     }
//
//     /// Creates a new `i64` counter with a given name and customized with passed options.
//     fn new_i64_counter<S: Into<String>>(
//         &self,
//         name: S,
//         opts: api::MetricOptions,
//     ) -> Result<Self::I64Counter, Error> {
//         let api::MetricOptions {
//             description,
//             unit,
//             keys,
//             alternate: _alternative,
//         } = opts;
//         let counter_opts = self.build_opts(name.into(), unit, description);
//         let labels = prometheus::convert_labels(&keys);
//         let counter = prometheus::IntCounterVec::new(counter_opts, &labels)?;
//         self.registry.register(Box::new(counter.clone()))?;
//
//         Ok(counter)
//     }
//
//     /// Creates a new `f64` counter with a given name and customized with passed options.
//     fn new_f64_counter<S: Into<String>>(
//         &self,
//         name: S,
//         opts: api::MetricOptions,
//     ) -> Result<Self::F64Counter, Error> {
//         let api::MetricOptions {
//             description,
//             unit,
//             keys,
//             alternate: _alternative,
//         } = opts;
//         let counter_opts = self.build_opts(name.into(), unit, description);
//         let labels = prometheus::convert_labels(&keys);
//         let counter = prometheus::CounterVec::new(counter_opts, &labels)?;
//         self.registry.register(Box::new(counter.clone()))?;
//
//         Ok(counter)
//     }
//
//     /// Creates a new `i64` gauge with a given name and customized with passed options.
//     fn new_i64_observer<S: Into<String>>(
//         &self,
//         name: S,
//         opts: api::MetricOptions,
//     ) -> Result<Self::I64Observer, Error> {
//         let api::MetricOptions {
//             description,
//             unit,
//             keys,
//             alternate: _alternative,
//         } = opts;
//         let gauge_opts = self.build_opts(name.into(), unit, description);
//         let labels = prometheus::convert_labels(&keys);
//         let gauge = prometheus::IntGaugeVec::new(gauge_opts, &labels)?;
//         self.registry.register(Box::new(gauge.clone()))?;
//
//         Ok(gauge)
//     }
//
//     /// Creates a new `f64` gauge with a given name and customized with passed options.
//     fn new_f64_observer<S: Into<String>>(
//         &self,
//         name: S,
//         opts: api::MetricOptions,
//     ) -> Result<Self::F64Observer, Error> {
//         let api::MetricOptions {
//             description,
//             unit,
//             keys,
//             alternate: _alternative,
//         } = opts;
//         let gauge_opts = self.build_opts(name.into(), unit, description);
//         let labels = prometheus::convert_labels(&keys);
//         let gauge = prometheus::GaugeVec::new(gauge_opts, &labels)?;
//         self.registry.register(Box::new(gauge.clone()))?;
//
//         Ok(gauge)
//     }
//
//     /// Creates a new `i64` measure with a given name and customized with passed options.
//     fn new_i64_measure<S: Into<String>>(
//         &self,
//         name: S,
//         opts: api::MetricOptions,
//     ) -> Result<Self::I64Measure, Error> {
//         let api::MetricOptions {
//             description,
//             unit,
//             keys,
//             alternate: _alternative,
//         } = opts;
//         let common_opts = self.build_opts(name.into(), unit, description);
//         let labels = prometheus::convert_labels(&keys);
//         let histogram_opts = prometheus::HistogramOpts::from(common_opts);
//         let histogram = prometheus::HistogramVec::new(histogram_opts, &labels)?;
//         self.registry.register(Box::new(histogram.clone()))?;
//
//         Ok(prometheus::IntMeasure::new(histogram))
//     }
//
//     /// Creates a new `f64` measure with a given name and customized with passed options.
//     fn new_f64_measure<S: Into<String>>(
//         &self,
//         name: S,
//         opts: api::MetricOptions,
//     ) -> Result<Self::F64Measure, Error> {
//         let api::MetricOptions {
//             description,
//             unit,
//             keys,
//             alternate: _alternative,
//         } = opts;
//         let common_opts = self.build_opts(name.into(), unit, description);
//         let labels = prometheus::convert_labels(&keys);
//         let histogram_opts = prometheus::HistogramOpts::from(common_opts);
//         let histogram = prometheus::HistogramVec::new(histogram_opts, &labels)?;
//         self.registry.register(Box::new(histogram.clone()))?;
//
//         Ok(histogram)
//     }
//
//     /// Records a batch of measurements.
//     fn record_batch<M: IntoIterator<Item = api::Measurement<Self::LabelSet>>>(
//         &self,
//         label_set: &Self::LabelSet,
//         measurements: M,
//     ) {
//         for measure in measurements.into_iter() {
//             let instrument = measure.instrument();
//             instrument.record_one(measure.into_value(), &label_set);
//         }
//     }
// }

/// record maintains the state of one metric instrument.  Due
/// the use of lock-free algorithms, there may be more than one
/// `record` in existence at a time, although at most one can
/// be referenced from the `SDK.current` map.
struct Record {
    // // refMapped keeps track of refcounts and the mapping state to the
// // SDK.current map.
// refMapped refcountMapped
//
// // updateCount is incremented on every Update.
// updateCount int64
//
// // collectedCount is set to updateCount on collection,
// // supports checking for no updates during a round.
// collectedCount int64
//
// // storage is the stored label set for this record,
// // except in cases where a label set is shared due to
// // batch recording.
// storage label.Set
//
// // labels is the processed label set for this record.
// // this may refer to the `storage` field in another
// // record if this label set is shared resulting from
// // `RecordBatch`.
// labels *label.Set
//
// // sortSlice has a single purpose - as a temporary
// // place for sorting during labels creation to avoid
// // allocation.
// sortSlice label.Sortable
//
// // inst is a pointer to the corresponding instrument.
// inst *syncInstrument
//
// // recorder implements the actual RecordOne() API,
// // depending on the type of aggregation.  If nil, the
// // metric was disabled by the exporter.
// recorder export.Aggregator
}
