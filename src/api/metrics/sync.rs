//! Async metrics
use crate::api::metrics::{sdk_api, Number};
use crate::api::KeyValue;
use std::fmt;
use std::sync::Arc;

/// TODO
#[derive(Debug)]
pub struct Measurement {
    // number needs to be aligned for 64-bit atomic operations.
    pub(crate) number: Number,
    pub(crate) instrument: Arc<dyn sdk_api::SyncInstrument>,
}

/// TODO
#[derive(Debug)]
pub struct Observation {
    number: Number,
    instrument: Arc<dyn sdk_api::AsyncInstrument>,
}

impl Observation {
    /// TODO
    pub fn number(&self) -> &Number {
        &self.number
    }
    /// TODO
    pub fn instrument(&self) -> &Arc<dyn sdk_api::AsyncInstrument> {
        &self.instrument
    }
}

/// TODO
pub struct F64ObserverResult {
    instrument: Arc<dyn sdk_api::AsyncInstrument>,
    f: fn(&[KeyValue], &[Observation]),
}

impl F64ObserverResult {
    /// TODO
    pub fn new(
        instrument: Arc<dyn sdk_api::AsyncInstrument>,
        f: fn(&[KeyValue], &[Observation]),
    ) -> Self {
        F64ObserverResult { instrument, f }
    }
}

impl fmt::Debug for F64ObserverResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("F64ObserverResult")
            .field("instrument", &self.instrument)
            .field("f", &"fn(&[KeyValue], &[Observation])")
            .finish()
    }
}

impl F64ObserverResult {
    /// TODO
    pub fn observe(&self, value: f64, labels: &[KeyValue]) {
        (self.f)(
            labels,
            &[Observation {
                number: value.into(),
                instrument: self.instrument.clone(),
            }],
        )
    }
}

/// TODO
pub enum AsyncRunner {
    /// TODO
    F64(Box<dyn Fn(F64ObserverResult) + Send + Sync + 'static>),
}

impl AsyncRunner {
    /// TODO
    pub fn run(
        &self,
        instrument: Arc<dyn sdk_api::AsyncInstrument>,
        f: fn(&[KeyValue], &[Observation]),
    ) {
        match self {
            AsyncRunner::F64(run) => run(F64ObserverResult::new(instrument, f)),
        }
    }
}

impl fmt::Debug for AsyncRunner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AsyncRunner::F64(_) => f
                .debug_struct("AsyncRunner")
                .field("closure", &"Fn(F64ObserverResult)")
                .finish(),
        }
    }
}
