use crate::api::metrics::{
    sdk_api, Descriptor, InstrumentKind, Measurement, Meter, Number, NumberKind,
};
use crate::api::KeyValue;
use std::marker;
use std::sync::Arc;

/// TODO
#[derive(Debug)]
pub struct ValueRecorder<T> {
    instrument: Arc<dyn sdk_api::SyncInstrument>,
    _marker: marker::PhantomData<T>,
}

impl<T> ValueRecorder<T>
where
    T: Into<Number>,
{
    /// TODO
    pub fn bind<'a>(&self, labels: &'a [KeyValue]) -> BoundValueRecorder<'a> {
        BoundValueRecorder { labels }
    }

    /// TODO
    pub fn measurement(&self, value: T) -> Measurement {
        Measurement {
            number: value.into(),
            instrument: self.instrument.clone(),
        }
    }
}

/// TODO
#[derive(Debug)]
pub struct BoundValueRecorder<'a> {
    labels: &'a [KeyValue],
}

/// TODO
#[derive(Debug)]
pub struct ValueRecorderBuilder<'a, T> {
    pub(crate) meter: &'a Meter,
    pub(crate) descriptor: Descriptor,
    pub(crate) _marker: marker::PhantomData<T>,
}

impl<'a, T> ValueRecorderBuilder<'a, T> {
    /// TODO
    pub fn new(
        meter: &'a Meter,
        name: String,
        instrument_kind: InstrumentKind,
        number_kind: NumberKind,
    ) -> Self {
        ValueRecorderBuilder {
            meter,
            descriptor: Descriptor::new(name, meter.name.clone(), instrument_kind, number_kind),
            _marker: marker::PhantomData,
        }
    }

    /// TODO
    pub fn with_description<S: Into<String>>(mut self, description: S) -> Self {
        self.descriptor.config.description = Some(description.into());
        self
    }

    /// TODO
    pub fn init(self) -> ValueRecorder<T> {
        ValueRecorder {
            instrument: self.meter.new_sync_instrument(self.descriptor).unwrap(),
            _marker: marker::PhantomData,
        }
    }
}
