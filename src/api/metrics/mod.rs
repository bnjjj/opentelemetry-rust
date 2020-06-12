//! # OpenTelemetry Metrics API

use crate::api::{Context, KeyValue};
use std::error::Error;
use std::fmt;
use std::io;
use std::marker;
use std::result;
use std::sync::{Arc, PoisonError, TryLockError};
use thiserror::Error;

mod async_instrument;
mod config;
mod counter;
mod descriptor;
pub mod noop;
mod number;
pub mod registry;
pub mod sdk_api;
mod sync_instrument;
mod value_observer;
mod value_recorder;

pub use async_instrument::{AsyncRunner, Observation, ObserverResult};
pub use config::Config;
pub use counter::{BoundCounter, Counter, CounterBuilder};
pub use descriptor::Descriptor;
pub use number::{Number, NumberKind};
pub use sync_instrument::Measurement;
pub use value_observer::{ValueObserver, ValueObserverBuilder};
pub use value_recorder::{BoundValueRecorder, ValueRecorder, ValueRecorderBuilder};

/// TODO
pub type Result<T> = result::Result<T, MetricsError>;

/// TODO
#[derive(Error, Debug)]
pub enum MetricsError {
    /// TODO
    #[error("metrics error: {0}")]
    Other(String),
    /// TODO
    #[error("metrics error: {0}")]
    StdError(#[source] Box<dyn Error + Send + 'static>),
    /// TODO
    #[error("the requested quantile is out of range")]
    InvalidQuantile,
    /// TODO
    #[error("NaN value is an invalid input")]
    NaNInput,
    /// TODO
    #[error("negative value is out of range for this instrument")]
    NegativeInput,
    /// TODO
    #[error("unknown async runner type: {0} (reported once)")]
    InvalidAsyncRunner(String),
    /// TODO
    #[error("cannot merge {0}, inconsistent aggregator types")]
    InconsistentMergeError(String),
    /// TODO
    #[error("io error: {0}")]
    IO(#[from] io::Error),
    /// TODO
    #[error("no data collected by this aggregator")]
    NoDataCollected,
}

impl<T> From<TryLockError<T>> for MetricsError {
    fn from(err: TryLockError<T>) -> Self {
        MetricsError::Other(err.to_string())
    }
}

impl<T> From<PoisonError<T>> for MetricsError {
    fn from(err: PoisonError<T>) -> Self {
        MetricsError::Other(err.to_string())
    }
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
        F: Fn(ObserverResult<f64>) + Send + Sync + 'static,
    {
        ValueObserverBuilder {
            meter: self,
            descriptor: Descriptor::new(
                name.into(),
                self.name.clone(),
                InstrumentKind::ValueObserver,
                NumberKind::F64,
            ),
            runner: AsyncRunner::F64(Box::new(callback)),
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
    pub fn u64_value_recorder<T>(&self, name: T) -> ValueRecorderBuilder<u64>
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
                NumberKind::U64,
            ),
            _marker: marker::PhantomData,
        }
    }

    ///TODO
    pub fn u64_counter<T>(&self, name: T) -> CounterBuilder<u64>
    where
        Self: Sized,
        T: Into<String>,
    {
        CounterBuilder::new(
            self,
            name.into(),
            InstrumentKind::ValueRecorder,
            NumberKind::U64,
        )
    }

    /// TODO
    fn new_sync_instrument(
        &self,
        descriptor: Descriptor,
    ) -> Result<Arc<dyn sdk_api::SyncInstrument>> {
        self.core.new_sync_instrument(descriptor)
    }

    /// TODO
    fn new_async_instrument(
        &self,
        descriptor: Descriptor,
        runner: AsyncRunner,
    ) -> Result<Arc<dyn sdk_api::AsyncInstrument>> {
        self.core.new_async_instrument(descriptor, runner)
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
#[derive(Clone, Debug, PartialEq, Hash)]
pub enum InstrumentKind {
    /// TODO
    ValueObserver,
    /// TODO
    ValueRecorder,
    /// TODO
    Counter,
    /// TODO
    SumObserver,
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
