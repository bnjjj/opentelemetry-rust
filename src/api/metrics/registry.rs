//! Metrics Registry API
use crate::api::metrics::{sdk_api::MeterCore, Meter, MeterProvider};
use std::sync::Arc;

/// TODO
pub fn meter_provider(core: Arc<dyn MeterCore + Send + Sync>) -> RegistryMeterProvider {
    RegistryMeterProvider(core)
}

/// TODO
#[derive(Debug, Clone)]
pub struct RegistryMeterProvider(Arc<dyn MeterCore + Send + Sync>);

impl MeterProvider for RegistryMeterProvider {
    fn meter(&self, name: &str) -> Meter {
        Meter::new(name, self.0.clone())
    }
}
