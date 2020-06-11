use crate::api::metrics::{registry, Result};
use crate::sdk::{
    export::metrics::{AggregationSelector, CheckpointSet, LockedIntegrator, Record},
    metrics::{
        accumulator,
        integrators::{self, SimpleIntegrator},
        Accumulator, ErrorHandler,
    },
    Resource,
};
use std::sync::Arc;
use std::time::{Duration, SystemTime};

/// TODO
pub fn pull(selector: Box<dyn AggregationSelector + Send + Sync>) -> PullControllerBuilder {
    PullControllerBuilder::with_selector(selector)
}

/// TODO
#[derive(Debug)]
pub struct PullController {
    accumulator: Accumulator,
    integrator: Arc<SimpleIntegrator>,
    provider: registry::RegistryMeterProvider,
    period: Duration,
    last_collect: SystemTime,
    error_handler: Option<ErrorHandler>,
}

impl PullController {
    /// TODO
    pub fn provider(&self) -> registry::RegistryMeterProvider {
        self.provider.clone()
    }

    /// TODO
    pub fn collect(&self) {
        if self
            .last_collect
            .elapsed()
            .map_or(true, |elapsed| elapsed > self.period)
        {
            match self.integrator.lock() {
                Ok(mut locked_integrator) => {
                    self.accumulator.0.collect(&mut locked_integrator);
                }
                Err(err) => {
                    if let Some(error_handler) = self.error_handler.as_ref() {
                        error_handler.call(err);
                    }
                }
            }
        }
    }
}

impl CheckpointSet for PullController {
    fn try_for_each(
        &mut self,
        f: &mut dyn FnMut(&Record) -> Result<()>,
    ) -> crate::api::metrics::Result<()> {
        self.integrator
            .lock()
            .and_then(|mut locked_integrator| locked_integrator.checkpoint_set().try_for_each(f))
    }
}

/// TODO
#[derive(Debug)]
pub struct PullControllerBuilder {
    /// TODO
    selector: Box<dyn AggregationSelector + Send + Sync>,
    /// Resource is the OpenTelemetry resource associated with all Meters
    /// created by the Controller.
    resource: Arc<Resource>,

    /// Stateful causes the controller to maintain state across
    /// collection events, so that records in the exported
    /// checkpoint set are cumulative.
    stateful: bool,

    /// CachePeriod is the period which a recently-computed result
    /// will be returned without gathering metric data again.
    ///
    /// If the period is zero, caching of the result is disabled.
    /// The default value is 10 seconds.
    cache_period: Duration,

    /// Error handler to use for this controller
    error_handler: Option<ErrorHandler>,
}

impl PullControllerBuilder {
    /// TODO
    pub fn with_selector(selector: Box<dyn AggregationSelector + Send + Sync>) -> Self {
        PullControllerBuilder {
            selector,
            resource: Arc::new(Resource::default()),
            stateful: false,
            cache_period: Duration::from_secs(10),
            error_handler: None,
        }
    }

    /// TODO
    pub fn build(self) -> PullController {
        let integrator = Arc::new(integrators::simple(self.selector, self.stateful));

        let accumulator = accumulator(integrator.clone())
            .with_resource(self.resource)
            .build();
        let provider = registry::meter_provider(Arc::new(accumulator.clone()));

        PullController {
            accumulator,
            integrator,
            provider,
            period: self.cache_period,
            last_collect: SystemTime::now(),
            error_handler: self.error_handler,
        }
    }
}
