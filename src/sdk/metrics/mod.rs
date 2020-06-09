//! # OpenTelemetry Metrics SDK
use crate::api::metrics::{
    sdk_api::{self, BoundSyncInstrument as _},
    AsyncRunner, Descriptor, Measurement, MetricsError, Number, NumberKind, Observation, Result,
};
use crate::api::{labels, Context, KeyValue};
use crate::sdk::{
    export::{
        self,
        metrics::{aggregator, Aggregator, Integrator, LockedIntegrator},
    },
    resource::Resource,
};
use std::any::Any;
use std::cmp::Ordering;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};

pub mod aggregators;
pub mod controllers;
pub mod integrators;
pub mod selectors;

pub use controllers::{PushController, PushControllerWorker};

///TODO
#[derive(Clone)]
pub struct ErrorHandler(Arc<dyn Fn(MetricsError) + Send + Sync>);

impl ErrorHandler {
    /// TODO
    pub fn call(&self, err: MetricsError) {
        self.0(err)
    }
}

impl fmt::Debug for ErrorHandler {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ErrorHandler")
            .field("closure", &"Fn(MetricsError)")
            .finish()
    }
}

impl ErrorHandler {
    /// TODO
    pub fn new<F>(handler: F) -> Self
    where
        F: Fn(MetricsError) + Send + Sync + 'static,
    {
        ErrorHandler(Arc::new(handler))
    }
}

/// TODO
pub fn accumulator(integrator: Arc<dyn Integrator + Send + Sync>) -> AccumulatorBuilder {
    AccumulatorBuilder {
        integrator,
        error_handler: None,
        push: false,
        resource: None,
    }
}

/// TODO
#[derive(Debug)]
pub struct AccumulatorBuilder {
    integrator: Arc<dyn Integrator + Send + Sync>,
    error_handler: Option<ErrorHandler>,
    push: bool,
    resource: Option<Arc<Resource>>,
}

impl AccumulatorBuilder {
    /// TODO
    pub fn with_error_handler(self, error_handler: ErrorHandler) -> Self {
        AccumulatorBuilder {
            error_handler: Some(error_handler),
            ..self
        }
    }

    /// TODO
    pub fn with_push(self, push: bool) -> Self {
        AccumulatorBuilder { push, ..self }
    }

    /// TODO
    pub fn with_resource(self, resource: Arc<Resource>) -> Self {
        AccumulatorBuilder {
            resource: Some(resource),
            ..self
        }
    }

    /// TODO
    pub fn build(self) -> Accumulator {
        Accumulator(Arc::new(AccumulatorCore::new(
            self.integrator,
            self.error_handler,
        )))
    }
}

/// TODO
#[derive(Debug, Clone)]
pub struct Accumulator(Arc<AccumulatorCore>);

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct MapKey {
    descriptor_hash: u64,
    ordered_hash: u64,
}

#[derive(Debug)]
struct AsyncInstrumentState {
    // /// runnerMap keeps the set of runners that will run each
    // /// collection interval.  Singletons are entered with a real
    // /// instrument each, batch observers are entered with a nil
    // /// instrument, ensuring that when a singleton callback is used
    // /// repeatedly, it is executed repeatedly in the interval, while
    // /// when a batch callback is used repeatedly, it only executes
    // /// once per interval.
    // runner_map: HashMap<Runner, Any>,
    /// runners maintains the set of runners in the order they were
    /// registered.
    runners: Vec<(AsyncRunner, Arc<dyn sdk_api::AsyncInstrument + Send + Sync>)>,
    // instruments maintains the set of instruments in the order
    // they were registered.
}

fn collect_async(labels: &[KeyValue], observations: &[Observation]) {
    let labels = labels::Set::from(labels);

    for observation in observations {
        if let Some(instrument) = observation
            .instrument()
            .as_any()
            .downcast_ref::<AsyncInstrument>()
        {
            instrument.observe(observation.number(), &labels)
        }
    }
}

impl AsyncInstrumentState {
    fn run(&self) {
        for (runner, instrument) in self.runners.iter() {
            // // The runner must be a single or batch runner, no
            // // other implementations are possible because the
            // // interface has un-exported methods.
            // if let Some(single_runner) = rp.as_any().downcast_ref::<AsyncSingleRunner>() {
            //     single_runner.run(rp.instrument, &collector.collect_async);
            //     continue;
            // }
            // if let Some(multi_runner) = rp.as_any().downcast_ref::<AsyncBatchRunner>() {
            //     multi_runner.run(rp.instrument, &collector.collect_async);
            //     continue;
            // }
            //
            // if let Some(error_handler) = collector.error_handler() {
            //     error_handler.call(MetricsError::InvalidAsyncRunner(format!("{:?}", rp)))
            // }
            runner.run(instrument.clone(), collect_async)
        }
    }
}

/// TODO
#[derive(Debug)]
struct AccumulatorCore {
    // current maps `mapkey` to *record.
    // current: dashmap::DashMap<MapKey, Record>,
    current: flurry::HashMap<MapKey, Arc<Record>>,
    //
    // // asyncInstruments is a set of
    async_instruments: Mutex<AsyncInstrumentState>,
    // asyncContext     context.Context
    //
    // // currentEpoch is the current epoch number. It is
    // // incremented in `Collect()`.
    current_epoch: Number,
    //
    // // integrator is the configured integrator+configuration.
    integrator: Arc<dyn Integrator + Send + Sync>,
    //
    // // collectLock prevents simultaneous calls to Collect().
    // collectLock sync.Mutex
    //
    // // errorHandler supports delivering errors to the user.
    error_handler: Option<ErrorHandler>,
    //
    // // asyncSortSlice has a single purpose - as a temporary
    // // place for sorting during labels creation to avoid
    // // allocation.  It is cleared after use.
    // asyncSortSlice label.Sortable
    //
    // // resource is applied to all records in this Accumulator.
    resource: Arc<Resource>,
}

impl AccumulatorCore {
    fn new(
        integrator: Arc<dyn Integrator + Send + Sync>,
        error_handler: Option<ErrorHandler>,
    ) -> Self {
        AccumulatorCore {
            current: flurry::HashMap::new(),
            async_instruments: Mutex::new(AsyncInstrumentState {
                runners: Vec::default(),
            }),
            current_epoch: NumberKind::U64.zero(),
            integrator,
            error_handler,
            resource: Arc::new(Resource::default()),
        }
    }

    fn register(
        &self,
        instrument: Arc<dyn sdk_api::AsyncInstrument + Send + Sync>,
        runner: AsyncRunner,
    ) -> Result<()> {
        self.async_instruments
            .lock()
            .map_err(|lock_err| MetricsError::Other(lock_err.to_string()))
            .map(|mut async_instruments| {
                async_instruments.runners.push((runner, instrument));
            })
    }

    fn collect(&self, locked_integrator: &mut dyn LockedIntegrator) -> usize {
        let mut checkpointed = self.observe_async_instruments(locked_integrator);
        checkpointed += self.collect_sync_instruments(locked_integrator);
        self.current_epoch.add(&NumberKind::U64, &1u64.into());

        checkpointed
    }

    fn observe_async_instruments(&self, locked_integrator: &mut dyn LockedIntegrator) -> usize {
        self.async_instruments
            .lock()
            .map_or(0, |async_instruments| {
                let mut async_collected = 0;
                // self.async_context = cx;

                async_instruments.run();
                // m.asyncContext = None;

                for (_runner, instrument) in &async_instruments.runners {
                    if let Some(a) = instrument.as_any().downcast_ref::<AsyncInstrument>() {
                        async_collected += self.checkpoint_async(a, locked_integrator);
                    }
                }

                async_collected
            })
    }

    fn collect_sync_instruments(&self, locked_integrator: &mut dyn LockedIntegrator) -> usize {
        let mut checkpointed = 0;
        let current_pin = self.current.pin();

        for (key, value) in current_pin.iter() {
            let mods = &value.update_count;
            let coll = &value.collected_count;

            if mods.partial_cmp(&NumberKind::U64, coll) != Some(Ordering::Equal) {
                // Updates happened in this interval,
                // checkpoint and continue.
                checkpointed += self.checkpoint_record(value, locked_integrator);
                value.collected_count.assign(&NumberKind::U64, mods);
            } else {
                // Having no updates since last collection, try to remove if
                // there are no bound handles
                if Arc::strong_count(&value) == 1 {
                    current_pin.remove(key);

                    // There's a potential race between loading collected count and
                    // loading the strong count in this function.  Since this is the
                    // last we'll see of this record, checkpoint.
                    if mods.partial_cmp(&NumberKind::U64, coll) != Some(Ordering::Equal) {
                        checkpointed += self.checkpoint_record(value, locked_integrator);
                    }
                }
            }
        }

        checkpointed
    }

    fn checkpoint(
        &self,
        descriptor: &Descriptor,
        recorder: Option<&Arc<dyn Aggregator + Send + Sync>>,
        labels: &labels::Set,
        locked_integrator: &mut dyn LockedIntegrator,
    ) -> usize {
        match recorder {
            None => 0,
            Some(recorder) => {
                recorder.checkpoint(descriptor);

                let export_record =
                    export::metrics::record(descriptor, labels, &self.resource, recorder);
                if let Err(_err) = locked_integrator.process(export_record) {
                    todo!()
                    // global::handle(err)
                }

                1
            }
        }
    }

    fn checkpoint_record(
        &self,
        record: &Record,
        locked_integrator: &mut dyn LockedIntegrator,
    ) -> usize {
        self.checkpoint(
            &record.instrument.instrument.descriptor,
            record.recorder.as_ref(),
            &record.labels,
            locked_integrator,
        )
    }

    fn checkpoint_async(
        &self,
        instrument: &AsyncInstrument,
        locked_integrator: &mut dyn LockedIntegrator,
    ) -> usize {
        instrument.recorders.lock().map_or(0, |mut recorders| {
            let mut checkpointed = 0;
            match recorders.as_mut() {
                None => return checkpointed,
                Some(recorders) => {
                    recorders.retain(|_key, label_recorder| {
                        let epoch_diff = self
                            .current_epoch
                            .partial_cmp(&NumberKind::U64, &label_recorder.observed_epoch.into());
                        if epoch_diff == Some(Ordering::Equal) {
                            checkpointed += self.checkpoint(
                                // m.asyncContext,
                                &instrument.instrument.descriptor,
                                label_recorder.recorder.as_ref(),
                                &label_recorder.labels,
                                locked_integrator,
                            )
                        }

                        // Retain if this is not second collection cycle with no
                        // observations for this labelset.
                        epoch_diff == Some(Ordering::Greater)
                    });
                }
            }
            if recorders.as_ref().map_or(false, |map| map.is_empty()) {
                *recorders = None;
            }

            checkpointed
        })
    }
}

///TODO
#[derive(Debug, Clone)]
pub struct SyncInstrument {
    instrument: Arc<Instrument>,
}

impl SyncInstrument {
    /// TODO
    fn acquire_handle(&self, labels: &[KeyValue]) -> Arc<Record> {
        let mut hasher = DefaultHasher::new();
        self.instrument.descriptor.hash(&mut hasher);
        let descriptor_hash = hasher.finish();

        let distinct = labels::Distinct::from(labels);

        let mut hasher = DefaultHasher::new();
        distinct.hash(&mut hasher);
        let ordered_hash = hasher.finish();

        let map_key = MapKey {
            descriptor_hash,
            ordered_hash,
        };
        let current_pin = self.instrument.meter.0.current.pin();
        if let Some(existing_record) = current_pin.get(&map_key) {
            return existing_record.clone();
        }

        let record = Arc::new(Record {
            update_count: Number::default(),
            collected_count: Number::default(),
            labels: labels::Set::with_equivalent(distinct),
            instrument: self.clone(),
            recorder: self
                .instrument
                .meter
                .0
                .integrator
                .aggregation_selector()
                .aggregator_for(&self.instrument.descriptor),
        });
        current_pin.insert(map_key, record.clone());

        record
    }
}

impl sdk_api::SyncInstrument for SyncInstrument {
    fn bind<'a>(
        &self,
        labels: &'a [crate::api::KeyValue],
    ) -> Arc<dyn sdk_api::BoundSyncInstrument> {
        self.acquire_handle(labels)
    }
    fn record_one_with_context<'a>(
        &self,
        _cx: &crate::api::Context,
        _number: crate::api::metrics::Number,
        _labels: &'a [crate::api::KeyValue],
    ) {
        todo!()
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

// TODO
#[derive(Debug)]
struct LabeledRecorder {
    observed_epoch: u64,
    labels: labels::Set,
    recorder: Option<Arc<dyn Aggregator + Send + Sync>>,
}

///TODO
#[derive(Debug, Clone)]
pub struct AsyncInstrument {
    instrument: Arc<Instrument>,
    /// FIXME: this may not require Mutex if it is not accessed by multiple threads
    recorders: Arc<Mutex<Option<HashMap<u64, LabeledRecorder>>>>,
}

impl AsyncInstrument {
    fn observe(&self, number: &Number, labels: &labels::Set) {
        if let Err(_err) = aggregator::range_test(number, &self.instrument.descriptor) {
            todo!()
            // global::handle(err);
            // return;
        }
        if let Some(recorder) = self.get_recorder(labels) {
            if let Err(_err) = recorder.update(number, &self.instrument.descriptor) {
                todo!()
                // global.handle(err)
                // return
            }
        }
    }

    fn get_recorder(&self, labels: &labels::Set) -> Option<Arc<dyn Aggregator + Send + Sync>> {
        self.recorders.lock().map_or(None, |mut recorders| {
            let mut hasher = DefaultHasher::new();
            labels.equivalent().hash(&mut hasher);
            let label_hash = hasher.finish();
            if let Some(recorder) = recorders.as_mut().and_then(|rec| rec.get_mut(&label_hash)) {
                let current_epoch = self.instrument.meter.0.current_epoch.to_u64();
                if recorder.observed_epoch == current_epoch {
                    // last value wins for Observers, so if we see the same labels
                    // in the current epoch, we replace the old recorder
                    recorder.recorder = self
                        .instrument
                        .meter
                        .0
                        .integrator
                        .aggregation_selector()
                        .aggregator_for(&self.instrument.descriptor)
                } else {
                    recorder.observed_epoch = current_epoch;
                }
                // self.recorders.insert(labels.equivalent().hash_value(), recorder);
                // Does this need clone?
                return recorder.recorder.clone();
            }

            let recorder = self
                .instrument
                .meter
                .0
                .integrator
                .aggregation_selector()
                .aggregator_for(&self.instrument.descriptor);
            if recorders.is_none() {
                *recorders = Some(HashMap::new());
            }
            // This may store nil recorder in the map, thus disabling the
            // asyncInstrument for the labelset for good. This is intentional,
            // but will be revisited later.
            recorders.as_mut().unwrap().insert(
                label_hash,
                LabeledRecorder {
                    recorder: recorder.clone(),
                    labels: labels::Set::with_equivalent(labels.equivalent().clone()),
                    observed_epoch: self.instrument.meter.0.current_epoch.to_u64(),
                },
            );

            recorder
        })
    }
}

impl sdk_api::Instrument for AsyncInstrument {
    fn descriptor(&self) -> &str {
        "AsyncInstrument"
    }
}

impl sdk_api::AsyncInstrument for AsyncInstrument {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// record maintains the state of one metric instrument.  Due
/// the use of lock-free algorithms, there may be more than one
/// `record` in existence at a time, although at most one can
/// be referenced from the `Accumulator.current` map.
#[derive(Debug)]
struct Record {
    // refMapped keeps track of refcounts and the mapping state to the
    // Accumulator.current map.
    // refMapped refcountMapped

    // updateCount is incremented on every Update.
    update_count: Number,

    // collectedCount is set to updateCount on collection,
    // supports checking for no updates during a round.
    collected_count: Number,

    // storage is the stored label set for this record,
    // except in cases where a label set is shared due to
    // batch recording.
    // storage: labels::Set,

    // labels is the processed label set for this record.
    // this may refer to the `storage` field in another
    // record if this label set is shared resulting from
    // `RecordBatch`.
    labels: labels::Set,

    // sortSlice has a single purpose - as a temporary
    // place for sorting during labels creation to avoid
    // allocation.
    // sortSlice label.Sortable

    // inst is a pointer to the corresponding instrument.
    instrument: SyncInstrument,

    // recorder implements the actual RecordOne() API,
    // depending on the type of aggregation.  If nil, the
    // metric was disabled by the exporter.
    recorder: Option<Arc<dyn Aggregator + Send + Sync>>,
}

impl sdk_api::BoundSyncInstrument for Record {
    fn record_one_with_context<'a>(&self, cx: &Context, number: Number) {
        // check if the instrument is disabled according to the AggregationSelector.
        if let Some(recorder) = &self.recorder {
            if let Err(err) = aggregator::range_test(
                &number,
                &self.instrument.instrument.descriptor,
            )
            .and_then(|_| {
                recorder.update_with_context(cx, &number, &self.instrument.instrument.descriptor)
            }) {
                if let Some(error_handler) = &self.instrument.instrument.meter.0.error_handler {
                    error_handler.call(err);
                }
                return;
            }

            // Record was modified, inform the Collect() that things need
            // to be collected while the record is still mapped.
            self.update_count.add(&NumberKind::U64, &1u64.into());
        }
    }
}

///TODO
#[derive(Debug)]
pub struct Instrument {
    descriptor: Descriptor,
    meter: Accumulator,
}

impl sdk_api::MeterCore for Accumulator {
    fn new_sync_instrument(
        &self,
        descriptor: Descriptor,
    ) -> Result<Arc<dyn sdk_api::SyncInstrument>> {
        Ok(Arc::new(SyncInstrument {
            instrument: Arc::new(Instrument {
                descriptor,
                meter: self.clone(),
            }),
        }))
    }

    fn record_batch_with_context(
        &self,
        cx: &Context,
        labels: &[KeyValue],
        measurements: Vec<Measurement>,
    ) {
        // var labelsPtr *label.Set
        for measure in measurements.into_iter() {
            if let Some(instrument) = measure.instrument.as_any().downcast_ref::<SyncInstrument>() {
                let handle = instrument.acquire_handle(labels);

                // Re-use labels for the next measurement.
                // if i == 0 {
                //     labelsPtr = h.labels
                // }

                handle.record_one_with_context(cx, measure.number);
            }
        }
    }

    fn new_async_instrument(
        &self,
        descriptor: Descriptor,
        runner: AsyncRunner,
    ) -> Result<Arc<dyn sdk_api::AsyncInstrument>> {
        let instrument = Arc::new(AsyncInstrument {
            instrument: Arc::new(Instrument {
                descriptor,
                meter: self.clone(),
            }),
            recorders: Arc::new(Mutex::new(None)),
        });

        self.0.register(instrument.clone(), runner)?;

        Ok(instrument)
    }
}

impl Accumulator {}

// //!
// //! The metrics SDK supports producing diagnostic measurements
// //! using three basic kinds of `Instrument`s. "Metrics" are the thing being
// //! produced--mathematical, statistical summaries of certain observable
// //! behavior in the program. `Instrument`s are the devices used by the
// //! program to record observations about their behavior. Therefore, we use
// //! "metric instrument" to refer to a program object, allocated through the
// //! `Meter` struct, used for recording metrics. There are three distinct
// //! instruments in the Metrics API, commonly known as `Counter`s, `Gauge`s,
// //! and `Measure`s.
// use crate::api;
// use crate::exporter::metrics::prometheus;
// use std::borrow::Cow;
// use std::collections::HashMap;
//
// /// Collection of label key and value types.
// pub type LabelSet = HashMap<Cow<'static, str>, Cow<'static, str>>;
// impl api::LabelSet for LabelSet {}
//
// /// `Meter` implementation to create manage metric instruments and record
// /// batch measurements
// #[allow(missing_debug_implementations)]
// pub struct Meter {
//     registry: &'static prometheus::Registry,
//     component: &'static str,
// }
//
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
//     type I64Gauge = prometheus::IntGaugeVec;
//     /// This implementation of `api::Meter` produces `prometheus::GaugeVec;` instances.
//     type F64Gauge = prometheus::GaugeVec;
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
//     ) -> Self::I64Counter {
//         let api::MetricOptions {
//             description,
//             unit,
//             keys,
//             alternate: _alternative,
//         } = opts;
//         let counter_opts = self.build_opts(name.into(), unit, description);
//         let labels = prometheus::convert_labels(&keys);
//         let counter = prometheus::IntCounterVec::new(counter_opts, &labels).unwrap();
//         self.registry.register(Box::new(counter.clone())).unwrap();
//
//         counter
//     }
//
//     /// Creates a new `f64` counter with a given name and customized with passed options.
//     fn new_f64_counter<S: Into<String>>(
//         &self,
//         name: S,
//         opts: api::MetricOptions,
//     ) -> Self::F64Counter {
//         let api::MetricOptions {
//             description,
//             unit,
//             keys,
//             alternate: _alternative,
//         } = opts;
//         let counter_opts = self.build_opts(name.into(), unit, description);
//         let labels = prometheus::convert_labels(&keys);
//         let counter = prometheus::CounterVec::new(counter_opts, &labels).unwrap();
//         self.registry.register(Box::new(counter.clone())).unwrap();
//
//         counter
//     }
//
//     /// Creates a new `i64` gauge with a given name and customized with passed options.
//     fn new_i64_gauge<S: Into<String>>(&self, name: S, opts: api::MetricOptions) -> Self::I64Gauge {
//         let api::MetricOptions {
//             description,
//             unit,
//             keys,
//             alternate: _alternative,
//         } = opts;
//         let gauge_opts = self.build_opts(name.into(), unit, description);
//         let labels = prometheus::convert_labels(&keys);
//         let gauge = prometheus::IntGaugeVec::new(gauge_opts, &labels).unwrap();
//         self.registry.register(Box::new(gauge.clone())).unwrap();
//
//         gauge
//     }
//
//     /// Creates a new `f64` gauge with a given name and customized with passed options.
//     fn new_f64_gauge<S: Into<String>>(&self, name: S, opts: api::MetricOptions) -> Self::F64Gauge {
//         let api::MetricOptions {
//             description,
//             unit,
//             keys,
//             alternate: _alternative,
//         } = opts;
//         let gauge_opts = self.build_opts(name.into(), unit, description);
//         let labels = prometheus::convert_labels(&keys);
//         let gauge = prometheus::GaugeVec::new(gauge_opts, &labels).unwrap();
//         self.registry.register(Box::new(gauge.clone())).unwrap();
//
//         gauge
//     }
//
//     /// Creates a new `i64` measure with a given name and customized with passed options.
//     fn new_i64_measure<S: Into<String>>(
//         &self,
//         name: S,
//         opts: api::MetricOptions,
//     ) -> Self::I64Measure {
//         let api::MetricOptions {
//             description,
//             unit,
//             keys,
//             alternate: _alternative,
//         } = opts;
//         let common_opts = self.build_opts(name.into(), unit, description);
//         let labels = prometheus::convert_labels(&keys);
//         let histogram_opts = prometheus::HistogramOpts::from(common_opts);
//         let histogram = prometheus::HistogramVec::new(histogram_opts, &labels).unwrap();
//         self.registry.register(Box::new(histogram.clone())).unwrap();
//
//         prometheus::IntMeasure::new(histogram)
//     }
//
//     /// Creates a new `f64` measure with a given name and customized with passed options.
//     fn new_f64_measure<S: Into<String>>(
//         &self,
//         name: S,
//         opts: api::MetricOptions,
//     ) -> Self::F64Measure {
//         let api::MetricOptions {
//             description,
//             unit,
//             keys,
//             alternate: _alternative,
//         } = opts;
//         let common_opts = self.build_opts(name.into(), unit, description);
//         let labels = prometheus::convert_labels(&keys);
//         let histogram_opts = prometheus::HistogramOpts::from(common_opts);
//         let histogram = prometheus::HistogramVec::new(histogram_opts, &labels).unwrap();
//         self.registry.register(Box::new(histogram.clone())).unwrap();
//
//         histogram
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
