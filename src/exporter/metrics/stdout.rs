//! Stdout Metrics Exporter
use crate::api::labels;
use crate::api::{metrics, metrics::MetricsError};
use crate::global;
use crate::sdk::{
    export::metrics::{CheckpointSet, Exporter},
    metrics::{
        controllers::{self, PushController, PushControllerWorker},
        selectors::simple,
    },
};
use futures::Stream;
use std::fmt;
use std::io;
use std::time;

/// TODO
pub fn stdout<S, SO, I, IS, ISI>(spawn: S, interval: I) -> StdoutExporterBuilder<io::Stdout, S, I>
where
    S: Fn(PushControllerWorker) -> SO,
    I: Fn(time::Duration) -> IS,
    IS: Stream<Item = ISI> + Send + 'static,
{
    StdoutExporterBuilder::<io::Stdout, S, I>::new(spawn, interval)
}

/// TODO
#[derive(Debug)]
pub struct StdoutExporter<W> {
    writer: W,
    pretty_print: bool,
    do_not_print_time: bool,
    quantiles: Vec<f64>,
    label_encoder: Box<dyn labels::Encoder + Send + Sync>,
}

impl<W: fmt::Debug> Exporter for StdoutExporter<W> {
    fn export(&self, _checkpoint_set: &dyn CheckpointSet) -> Result<(), MetricsError> {
        todo!()
    }
}

/// TODO
#[derive(Debug)]
pub struct StdoutExporterBuilder<W, S, I> {
    spawn: S,
    interval: I,
    writer: W,
    pretty_print: bool,
    do_not_print_time: bool,
    quantiles: Option<Vec<f64>>,
    label_encoder: Option<Box<dyn labels::Encoder + Send + Sync>>,
}

impl<W, S, SO, I, IS, ISI> StdoutExporterBuilder<W, S, I>
where
    W: io::Write + fmt::Debug + Send + Sync + 'static,
    S: Fn(PushControllerWorker) -> SO,
    I: Fn(time::Duration) -> IS,
    IS: Stream<Item = ISI> + Send + 'static,
{
    fn new(spawn: S, interval: I) -> StdoutExporterBuilder<io::Stdout, S, I> {
        StdoutExporterBuilder {
            spawn,
            interval,
            writer: io::stdout(),
            pretty_print: false,
            do_not_print_time: false,
            quantiles: None,
            label_encoder: None,
        }
    }
    /// TODO
    pub fn with_writer<W2: io::Write>(self, writer: W2) -> StdoutExporterBuilder<W2, S, I> {
        StdoutExporterBuilder {
            spawn: self.spawn,
            interval: self.interval,
            writer,
            pretty_print: self.pretty_print,
            do_not_print_time: self.do_not_print_time,
            quantiles: self.quantiles,
            label_encoder: self.label_encoder,
        }
    }

    /// TODO
    pub fn with_pretty_print(self, pretty_print: bool) -> Self {
        StdoutExporterBuilder {
            pretty_print,
            ..self
        }
    }

    /// TODO
    pub fn with_do_not_print_time(self, do_not_print_time: bool) -> Self {
        StdoutExporterBuilder {
            do_not_print_time,
            ..self
        }
    }

    /// TODO
    pub fn with_quantiles(self, quantiles: Vec<f64>) -> Self {
        StdoutExporterBuilder {
            quantiles: Some(quantiles),
            ..self
        }
    }

    /// TODO
    pub fn with_label_encoder<E>(self, label_encoder: E) -> Self
    where
        E: labels::Encoder + Send + Sync + 'static,
    {
        StdoutExporterBuilder {
            label_encoder: Some(Box::new(label_encoder)),
            ..self
        }
    }

    /// TODO
    pub fn try_init(self) -> metrics::Result<PushController> {
        let (spawn, interval, exporter) = self.try_build()?;
        let controller = controllers::push(simple::Selector::Exact, exporter, spawn, interval)
            .with_stateful(true)
            .build();
        global::set_meter_provider(controller.provider());
        Ok(controller)
    }

    /// TODO
    fn try_build(self) -> metrics::Result<(S, I, StdoutExporter<W>)> {
        if let Some(quantiles) = self.quantiles.as_ref() {
            for q in quantiles {
                if *q < 0.0 || *q > 1.0 {
                    return Err(MetricsError::InvalidQuantile);
                }
            }
        }

        Ok((
            self.spawn,
            self.interval,
            StdoutExporter {
                writer: self.writer,
                pretty_print: self.pretty_print,
                do_not_print_time: self.do_not_print_time,
                quantiles: self.quantiles.unwrap_or(vec![0.5, 0.9, 0.99]),
                label_encoder: self.label_encoder.unwrap_or_else(labels::default_encoder),
            },
        ))
    }
}
