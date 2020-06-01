//! Stdout Metrics Exporter
use crate::api::labels;
use crate::api::{metrics, metrics::MetricsError};
use crate::global;
use crate::sdk::{
    export,
    metrics::{
        controllers::{self, PushController},
        selectors::simple,
    },
};
use std::fmt;
use std::io;

/// TODO
pub fn stdout() -> ExporterBuilder<io::Stdout> {
    ExporterBuilder::default()
}

/// TODO
#[derive(Debug)]
pub struct Exporter<W> {
    writer: W,
    pretty_print: bool,
    do_not_print_time: bool,
    quantiles: Vec<f64>,
    label_encoder: Box<dyn labels::Encoder>,
}

impl<W: fmt::Debug> export::metrics::Exporter for Exporter<W> {
    fn export(
        &self,
        checkpoint_set: &dyn export::metrics::CheckpointSet,
    ) -> Result<(), MetricsError> {
        todo!()
    }
}

/// TODO
#[derive(Debug)]
pub struct ExporterBuilder<W: io::Write> {
    writer: W,
    pretty_print: bool,
    do_not_print_time: bool,
    quantiles: Option<Vec<f64>>,
    label_encoder: Option<Box<dyn labels::Encoder>>,
}

impl Default for ExporterBuilder<io::Stdout> {
    fn default() -> Self {
        ExporterBuilder {
            writer: io::stdout(),
            pretty_print: false,
            do_not_print_time: false,
            quantiles: None,
            label_encoder: None,
        }
    }
}

impl<W> ExporterBuilder<W>
where
    W: io::Write + fmt::Debug + 'static,
{
    /// TODO
    pub fn with_writer<W2: io::Write>(self, writer: W2) -> ExporterBuilder<W2> {
        ExporterBuilder {
            writer,
            pretty_print: self.pretty_print,
            do_not_print_time: self.do_not_print_time,
            quantiles: self.quantiles,
            label_encoder: self.label_encoder,
        }
    }

    /// TODO
    pub fn with_pretty_print(self, pretty_print: bool) -> Self {
        ExporterBuilder {
            pretty_print,
            ..self
        }
    }

    /// TODO
    pub fn with_do_not_print_time(self, do_not_print_time: bool) -> Self {
        ExporterBuilder {
            do_not_print_time,
            ..self
        }
    }

    /// TODO
    pub fn with_quantiles(self, quantiles: Vec<f64>) -> Self {
        ExporterBuilder {
            quantiles: Some(quantiles),
            ..self
        }
    }

    /// TODO
    pub fn with_label_encoder<E>(self, label_encoder: E) -> Self
    where
        E: labels::Encoder + 'static,
    {
        ExporterBuilder {
            label_encoder: Some(Box::new(label_encoder)),
            ..self
        }
    }

    /// TODO
    pub fn try_init(self) -> metrics::Result<PushController> {
        let exporter = self.try_build()?;
        let controller = controllers::push(simple::Selector::Exact, exporter)
            .with_stateful(true)
            .build();
        global::set_meter_provider(controller.provider());
        Ok(controller)
    }

    /// TODO
    fn try_build(self) -> metrics::Result<Exporter<W>> {
        if let Some(quantiles) = self.quantiles.as_ref() {
            for q in quantiles {
                if *q < 0.0 || *q > 1.0 {
                    return Err(MetricsError::InvalidQuantile);
                }
            }
        }

        Ok(Exporter {
            writer: self.writer,
            pretty_print: self.pretty_print,
            do_not_print_time: self.do_not_print_time,
            quantiles: self.quantiles.unwrap_or(vec![0.5, 0.9, 0.99]),
            label_encoder: self.label_encoder.unwrap_or_else(labels::default_encoder),
        })
    }
}
