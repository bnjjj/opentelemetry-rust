use crate::api::metrics::registry;
use crate::sdk::{
    export::metrics::{AggregationSelector, Exporter, LockedIntegrator},
    metrics::{
        self,
        integrators::{self, SimpleIntegrator},
        Accumulator, ErrorHandler,
    },
    Resource,
};
use futures::{channel::mpsc, task, Future, Stream, StreamExt};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time;

lazy_static::lazy_static! {
    static ref DEFAULT_PUSH_PERIOD: time::Duration = time::Duration::from_secs(10);
}

/// Create a new `PushControllerBuilder`.
pub fn push<S, E, SP, SO, I, IO>(
    selector: S,
    exporter: E,
    spawn: SP,
    interval: I,
) -> PushControllerBuilder<SP, I>
where
    S: AggregationSelector + Send + Sync + 'static,
    E: Exporter + Send + Sync + 'static,
    SP: Fn(PushControllerWorker) -> SO,
    I: Fn(time::Duration) -> IO,
{
    PushControllerBuilder {
        selector: Box::new(selector),
        exporter: Box::new(exporter),
        spawn,
        interval,
        error_handler: None,
        resource: None,
        stateful: None,
        period: None,
        timeout: None,
    }
}

/// Organizes a periodic push of metric data.
#[derive(Debug)]
pub struct PushController {
    message_sender: Mutex<mpsc::Sender<PushMessage>>,
    provider: registry::RegistryMeterProvider,
}

#[derive(Debug)]
enum PushMessage {
    Tick,
    Shutdown,
}

/// The future which executes push controller work periodically. Can be run on a
/// passed in executor.
#[allow(missing_debug_implementations)]
pub struct PushControllerWorker {
    messages: Pin<Box<dyn Stream<Item = PushMessage> + Send>>,
    accumulator: Accumulator,
    integrator: Arc<SimpleIntegrator>,
    exporter: Box<dyn Exporter + Send + Sync>,
    error_handler: Option<ErrorHandler>,
    _timeout: time::Duration,
}

impl PushControllerWorker {
    fn on_tick(&mut self) {
        // ctx, cancel := context.WithTimeout(context.Background(), c.timeout)
        // defer cancel()

        if let Err(err) = self.integrator.lock().and_then(|mut locked_integrator| {
            self.accumulator.0.collect(&mut locked_integrator);
            self.exporter.export(locked_integrator.checkpoint_set())?;
            locked_integrator.finished_collection();
            Ok(())
        }) {
            if let Some(error_handler) = &self.error_handler {
                error_handler.call(err);
            }
        }
    }
}

impl Future for PushControllerWorker {
    type Output = ();
    fn poll(mut self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> task::Poll<Self::Output> {
        loop {
            match futures::ready!(self.messages.poll_next_unpin(cx)) {
                // Span batch interval time reached, export current spans.
                Some(PushMessage::Tick) => self.on_tick(),
                // Stream has terminated or processor is shutdown, return to finish execution.
                None | Some(PushMessage::Shutdown) => {
                    return task::Poll::Ready(());
                }
            }
        }
    }
}

impl Drop for PushControllerWorker {
    fn drop(&mut self) {
        // Try to push data one last time
        self.on_tick()
    }
}

impl PushController {
    /// The controller's meter provider.
    pub fn provider(&self) -> registry::RegistryMeterProvider {
        self.provider.clone()
    }
}

impl Drop for PushController {
    fn drop(&mut self) {
        if let Ok(mut sender) = self.message_sender.lock() {
            let _ = sender.try_send(PushMessage::Shutdown);
        }
    }
}

/// Configuration for building a new `PushController`.
#[derive(Debug)]
pub struct PushControllerBuilder<S, I> {
    selector: Box<dyn AggregationSelector + Send + Sync>,
    exporter: Box<dyn Exporter + Send + Sync>,
    spawn: S,
    interval: I,
    error_handler: Option<ErrorHandler>,
    resource: Option<Resource>,
    stateful: Option<bool>,
    period: Option<time::Duration>,
    timeout: Option<time::Duration>,
}

impl<S, SO, I, IS, ISI> PushControllerBuilder<S, I>
where
    S: Fn(PushControllerWorker) -> SO,
    I: Fn(time::Duration) -> IS,
    IS: Stream<Item = ISI> + Send + 'static,
{
    /// Configure the statefulness of this controller.
    pub fn with_stateful(self, stateful: bool) -> Self {
        PushControllerBuilder {
            stateful: Some(stateful),
            ..self
        }
    }

    /// Configure the period of this controller
    pub fn with_period(self, period: time::Duration) -> Self {
        PushControllerBuilder {
            period: Some(period),
            ..self
        }
    }

    /// Configure the error handler this controller will use.
    pub fn with_error_handler(self, error_handler: ErrorHandler) -> Self {
        PushControllerBuilder {
            error_handler: Some(error_handler),
            ..self
        }
    }

    /// Configure the resource used by this controller
    pub fn with_resource(self, resource: Resource) -> Self {
        PushControllerBuilder {
            resource: Some(resource),
            ..self
        }
    }

    /// Build a new `PushController` with this configuration.
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
        let accumulator = accumulator.build();
        let provider = registry::meter_provider(Arc::new(accumulator.clone()));

        let (message_sender, message_receiver) = mpsc::channel(256);
        let ticker =
            (self.interval)(self.period.unwrap_or(*DEFAULT_PUSH_PERIOD)).map(|_| PushMessage::Tick);

        (self.spawn)(PushControllerWorker {
            messages: Box::pin(futures::stream::select(message_receiver, ticker)),
            accumulator,
            integrator,
            exporter: self.exporter,
            error_handler: self.error_handler,
            // ch:           make(chan struct{}),
            _timeout: self.timeout.unwrap_or(*DEFAULT_PUSH_PERIOD),
            // clock:        controllerTime.RealClock{},
        });

        PushController {
            message_sender: Mutex::new(message_sender),
            provider,
        }
    }
}
