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
        todo!()
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
