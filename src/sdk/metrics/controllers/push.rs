use crate::api::metrics::{registry, MetricsError};
use crate::sdk::{
    export::metrics::{AggregationSelector, Exporter},
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
    static ref DEFAULT_ERROR_PERIOD: time::Duration = time::Duration::from_secs(10);
}

/// TODO
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

/// TODO
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

///TODO
#[allow(missing_debug_implementations)]
pub struct PushControllerWorker {
    messages: Pin<Box<dyn Stream<Item = PushMessage> + Send>>,
    provider: registry::RegistryMeterProvider,
    accumulator: Accumulator,
    integrator: Arc<SimpleIntegrator>,
    exporter: Box<dyn Exporter + Send + Sync>,
    error_handler: Option<ErrorHandler>,
    timeout: time::Duration,
}

impl Future for PushControllerWorker {
    type Output = ();
    fn poll(mut self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> task::Poll<Self::Output> {
        loop {
            match futures::ready!(self.messages.poll_next_unpin(cx)) {
                // Span batch interval time reached, export current spans.
                Some(PushMessage::Tick) => {
                    // ctx, cancel := context.WithTimeout(context.Background(), c.timeout)
                    // defer cancel()

                    self.integrator
                        .try_lock_inner(|_integrator| {
                            self.accumulator.0.collect();
                            //
                            // self.exporter.export(integrator.checkpoint_set())?;
                            // integrator.finished_collection();
                            //
                            // Ok(())
                        })
                        .map_err(|err| {
                            if let Some(error_handler) = &self.error_handler {
                                error_handler.call(err);
                            }
                        });
                }
                // Stream has terminated or processor is shutdown, return to finish execution.
                None | Some(PushMessage::Shutdown) => {
                    return task::Poll::Ready(());
                }
            }
        }
    }
}

impl PushController {
    /// TODO
    pub fn provider(&self) -> registry::RegistryMeterProvider {
        self.provider.clone()
    }
}

impl Drop for PushController {
    fn drop(&mut self) {
        if let Ok(mut sender) = self.message_sender.try_lock() {
            let _ = sender.try_send(PushMessage::Shutdown);
        }
    }
}

/// TODO
#[derive(Debug)]
pub struct PushControllerBuilder<S, I> {
    selector: Box<dyn AggregationSelector + Send + Sync>,
    exporter: Box<dyn Exporter + Send + Sync>,
    spawn: S,
    interval: I,
    error_handler: Option<ErrorHandler>,
    resource: Option<Arc<Resource>>,
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
    /// TODO
    pub fn with_stateful(self, stateful: bool) -> Self {
        PushControllerBuilder {
            stateful: Some(stateful),
            ..self
        }
    }

    pub fn with_error_handler<T>(self, error_handler: T) -> Self
    where
        T: Fn(MetricsError) + Send + Sync + 'static,
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
        let accumulator = accumulator.build();
        let provider = registry::meter_provider(Arc::new(accumulator.clone()));

        let (message_sender, message_receiver) = mpsc::channel(256);
        let ticker = (self.interval)(self.period.unwrap_or(DEFAULT_PUSH_PERIOD.clone()))
            .map(|_| PushMessage::Tick);

        (self.spawn)(PushControllerWorker {
            messages: Box::pin(futures::stream::select(message_receiver, ticker)),
            provider: provider.clone(),
            accumulator,
            integrator,
            exporter: self.exporter,
            error_handler: self.error_handler,
            // ch:           make(chan struct{}),
            timeout: self.timeout.unwrap_or(DEFAULT_PUSH_PERIOD.clone()),
            // clock:        controllerTime.RealClock{},
        });

        PushController {
            message_sender: Mutex::new(message_sender),
            provider,
        }
    }
}
