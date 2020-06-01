//! Async metrics
use crate::api::metrics::{sdk_api, Number};
use crate::api::KeyValue;
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
pub struct F64ObserverResult;

impl F64ObserverResult {
    /// TODO
    pub fn observe(&self, _value: f64, _labels: &[KeyValue]) {
        todo!()
    }
}
