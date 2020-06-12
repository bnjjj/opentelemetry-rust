use crate::api::metrics::{self, Meter, MeterProvider};
use std::sync::{Arc, RwLock};

lazy_static::lazy_static! {
    /// The global `Meter` provider singleton.
    static ref GLOBAL_METER_PROVIDER: RwLock<GlobalMeterProvider> = RwLock::new(GlobalMeterProvider::new(metrics::noop::NoopMeterProvider));
}

/// TODO
#[derive(Debug, Clone)]
pub struct GlobalMeterProvider {
    provider: Arc<dyn MeterProvider + Send + Sync>,
}

impl MeterProvider for GlobalMeterProvider {
    fn meter(&self, name: &str) -> Meter {
        self.provider.meter(name)
    }
}

impl GlobalMeterProvider {
    /// TODO
    pub fn new<P>(provider: P) -> Self
    where
        P: MeterProvider + Send + Sync + 'static,
    {
        GlobalMeterProvider {
            provider: Arc::new(provider),
        }
    }
}

/// TODO
pub fn set_meter_provider<P>(new_provider: P)
where
    P: metrics::MeterProvider + Send + Sync + 'static,
{
    let mut global_provider = GLOBAL_METER_PROVIDER
        .write()
        .expect("GLOBAL_METER_PROVIDER RwLock poisoned");
    *global_provider = GlobalMeterProvider::new(new_provider);
}

/// TODO
pub fn meter_provider() -> GlobalMeterProvider {
    GLOBAL_METER_PROVIDER
        .read()
        .expect("GLOBAL_METER_PROVIDER RwLock poisoned")
        .clone()
}

/// TODO
pub fn meter(name: &str) -> Meter {
    meter_provider().meter(name)
}
