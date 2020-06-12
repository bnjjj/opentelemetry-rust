use crate::api::metrics::{sdk_api, AsyncRunner, Descriptor, Meter};
use std::sync::Arc;

/// TODO
#[derive(Debug)]
pub struct ValueObserver<T> {
    instrument: Arc<dyn sdk_api::AsyncInstrument>,
    _marker: std::marker::PhantomData<T>,
}

/// TODO
#[derive(Debug)]
pub struct ValueObserverBuilder<'a, T> {
    pub(crate) meter: &'a Meter,
    pub(crate) descriptor: Descriptor,
    pub(crate) runner: AsyncRunner,
    pub(crate) _marker: std::marker::PhantomData<T>,
}

impl<T> ValueObserverBuilder<'_, T> {
    /// TODO
    pub fn with_description<S: Into<String>>(mut self, description: S) -> Self {
        self.descriptor.set_description(description.into());
        self
    }

    /// TODO
    pub fn init(self) -> ValueObserver<T> {
        ValueObserver {
            instrument: self
                .meter
                .new_async_instrument(self.descriptor, self.runner)
                .unwrap(),
            _marker: std::marker::PhantomData,
        }
    }
}
