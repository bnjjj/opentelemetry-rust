use crate::api::metrics::{registry, MetricsError, Result};
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

const DEFAULT_CACHE_DURATION: Duration = Duration::from_secs(10);

/// Returns a builder for creating a `PullController` with the configured and options.
pub fn pull(selector: Box<dyn AggregationSelector + Send + Sync>) -> PullControllerBuilder {
    PullControllerBuilder::with_selector(selector)
}

/// Controller manages access to an `Accumulator` and `Integrator`.  
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
    /// The provider for this controller
    pub fn provider(&self) -> registry::RegistryMeterProvider {
        self.provider.clone()
    }

    /// Collects all metrics if the last collected at time is past the current period
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

/// Configuration for a `PullController`.
#[derive(Debug)]
pub struct PullControllerBuilder {
    /// The selector used by the controller
    selector: Box<dyn AggregationSelector + Send + Sync>,
    /// Resource is the OpenTelemetry resource associated with all Meters created by
    /// the controller.
    resource: Option<Resource>,

    /// Stateful causes the controller to maintain state across collection events,
    /// so that records in the exported checkpoint set are cumulative.
    stateful: bool,

    /// CachePeriod is the period which a recently-computed result will be returned
    /// without gathering metric data again.
    ///
    /// If the period is zero, caching of the result is disabled. The default value
    /// is 10 seconds.
    cache_period: Option<Duration>,

    /// Error handler to use for this controller
    error_handler: Option<ErrorHandler>,
}

impl PullControllerBuilder {
    /// Configure the sector for this controller
    pub fn with_selector(selector: Box<dyn AggregationSelector + Send + Sync>) -> Self {
        PullControllerBuilder {
            selector,
            resource: None,
            stateful: false,
            cache_period: None,
            error_handler: None,
        }
    }

    /// Configure the resource for this controller
    pub fn with_resource(self, resource: Resource) -> Self {
        PullControllerBuilder {
            resource: Some(resource),
            ..self
        }
    }

    /// Configure the cache period for this controller
    pub fn with_cache_period(self, period: Duration) -> Self {
        PullControllerBuilder {
            cache_period: Some(period),
            ..self
        }
    }

    /// Configure the statefulness of this controller
    pub fn with_stateful(self, stateful: bool) -> Self {
        PullControllerBuilder { stateful, ..self }
    }

    /// Configure the error handler this controller will use
    pub fn with_error_handler<T>(self, f: T) -> Self
    where
        T: Fn(MetricsError) + Send + Sync + 'static,
    {
        PullControllerBuilder {
            error_handler: Some(ErrorHandler::new(f)),
            ..self
        }
    }

    /// Build a new `PushController` from the current configuration.
    pub fn build(self) -> PullController {
        let integrator = Arc::new(integrators::simple(self.selector, self.stateful));

        let accumulator = accumulator(integrator.clone())
            .with_resource(self.resource.unwrap_or_default())
            .build();
        let provider = registry::meter_provider(Arc::new(accumulator.clone()));

        PullController {
            accumulator,
            integrator,
            provider,
            period: self.cache_period.unwrap_or(DEFAULT_CACHE_DURATION),
            last_collect: SystemTime::now(),
            error_handler: self.error_handler,
        }
    }
}
