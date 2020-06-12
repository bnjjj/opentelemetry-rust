use crate::api::{
    metrics::{sdk_api, Number},
    KeyValue,
};
use std::marker;
use std::sync::Arc;

/// TODO
#[derive(Debug)]
pub struct Measurement {
    // number needs to be aligned for 64-bit atomic operations.
    pub(crate) number: Number,
    pub(crate) instrument: Arc<dyn sdk_api::SyncInstrument>,
}

/// Wrapper around a sdk-implemented sync instrument for a given type
#[derive(Debug)]
pub(crate) struct SyncInstrument<T> {
    instrument: Arc<dyn sdk_api::SyncInstrument>,
    _marker: marker::PhantomData<T>,
}

impl<T> SyncInstrument<T> {
    /// Create a new sync instrument from an sdk-implemented sync instrument
    pub(crate) fn new(instrument: Arc<dyn sdk_api::SyncInstrument>) -> Self {
        SyncInstrument {
            instrument,
            _marker: marker::PhantomData,
        }
    }

    /// Create a new bound sync instrument
    pub(crate) fn bind(&self, labels: &[KeyValue]) -> BoundSyncInstrument<T> {
        let bound_instrument = self.instrument.bind(labels);
        BoundSyncInstrument {
            bound_instrument,
            _marker: marker::PhantomData,
        }
    }

    /// Record a value directly to the underlying instrument
    pub(crate) fn direct_record(&self, number: Number, labels: &[KeyValue]) {
        self.instrument.record_one(number, labels)
    }

    /// Reference to the underlying sdk-implemented instrument
    pub(crate) fn instrument(&self) -> &Arc<dyn sdk_api::SyncInstrument> {
        &self.instrument
    }
}

/// Wrapper around a sdk-implemented sync bound instrument
#[derive(Debug)]
pub(crate) struct BoundSyncInstrument<T> {
    bound_instrument: Arc<dyn sdk_api::BoundSyncInstrument>,
    _marker: marker::PhantomData<T>,
}

impl<T> BoundSyncInstrument<T> {
    /// Record a value directly to the underlying instrument
    pub(crate) fn direct_record(&self, number: Number) {
        self.bound_instrument.record_one(number)
    }
}
