use crate::api::metrics::{registry, MetricsError};
use crate::sdk::{
    export::metrics::{AggregationSelector, Exporter},
    metrics::{self, integrators, Accumulator, ErrorHandler, Integrator},
    Resource,
};
use std::fmt;
use std::sync::Arc;
use std::time;

lazy_static::lazy_static! {
    static ref DEFAULT_PUSH_PERIOD: time::Duration = time::Duration::from_secs(10);
    static ref DEFAULT_ERROR_PERIOD: time::Duration = time::Duration::from_secs(10);
}

/// TODO
pub fn push<S, E>(selector: S, exporter: E) -> PushControllerBuilder
where
    S: AggregationSelector + 'static,
    E: Exporter + 'static,
{
    PushControllerBuilder::new(Box::new(selector), Box::new(exporter))
}

///TODO
#[derive(Debug)]
pub struct PushController {
    provider: registry::RegistryMeterProvider,
    accumulator: Arc<Accumulator>,
    integrator: Arc<dyn Integrator>,
    exporter: Box<dyn Exporter>,
    error_handler: Option<ErrorHandler>,
    period: time::Duration,
    timeout: time::Duration,
    // clock:        controllerTime.RealClock{},
}

impl PushController {
    /// TODO
    pub fn provider(&self) -> registry::RegistryMeterProvider {
        self.provider.clone()
    }
    /// TODO
    pub fn start(&self) {}
}

/// TODO
pub struct PushControllerBuilder {
    selector: Box<dyn AggregationSelector>,
    exporter: Box<dyn Exporter>,
    error_handler: Option<ErrorHandler>,
    resource: Option<Arc<Resource>>,
    stateful: Option<bool>,
    period: Option<time::Duration>,
    timeout: Option<time::Duration>,
}

impl fmt::Debug for PushControllerBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PushControllerBuilder")
            .field("selector", &self.selector)
            .field("exporter", &self.exporter)
            .field("error_handler", &"Fn(MetricsError)")
            .field("resource", &self.resource)
            .field("period", &self.period)
            .field("timeout", &self.timeout)
            .finish()
    }
}

impl PushControllerBuilder {
    pub fn new(selector: Box<dyn AggregationSelector>, exporter: Box<dyn Exporter>) -> Self {
        PushControllerBuilder {
            selector,
            exporter,
            error_handler: None,
            resource: None,
            stateful: None,
            period: None,
            timeout: None,
        }
    }

    /// TODO
    pub fn with_stateful(self, stateful: bool) -> Self {
        PushControllerBuilder {
            stateful: Some(stateful),
            ..self
        }
    }

    pub fn with_error_handler<T>(self, error_handler: T) -> Self
    where
        T: Fn(MetricsError) + 'static,
    {
        PushControllerBuilder {
            error_handler: Some(ErrorHandler::new(error_handler)),
            ..self
        }
    }

    /// TODO
    pub fn build(self) -> PushController {
        let integrator = integrators::simple(self.selector, self.stateful.unwrap_or(false));
        let integrator = Arc::new(integrator);
        let mut accumulator = metrics::accumulator(integrator.clone());

        if let Some(error_handler) = &self.error_handler {
            accumulator = accumulator.with_error_handler(error_handler.clone());
        }

        if let Some(resource) = self.resource {
            accumulator = accumulator.with_resource(resource);
        }
        let accumulator = Arc::new(accumulator.build());

        PushController {
            provider: registry::meter_provider(accumulator.clone()),
            accumulator,
            integrator,
            exporter: self.exporter,
            error_handler: self.error_handler,
            // ch:           make(chan struct{}),
            period: self.period.unwrap_or(DEFAULT_PUSH_PERIOD.clone()),
            timeout: self.timeout.unwrap_or(DEFAULT_PUSH_PERIOD.clone()),
            // clock:        controllerTime.RealClock{},
        }
    }
}
