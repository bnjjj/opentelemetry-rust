//! Stdout Metrics Exporter
use crate::api::labels;
use crate::api::{
    metrics,
    metrics::{MetricsError, Result},
};
use crate::global;
use crate::sdk::{
    export::metrics::{CheckpointSet, Count, Exporter, LastValue, Max, Min, Quantile, Sum},
    metrics::{
        aggregators::{
            ArrayAggregator, HistogramAggregator, LastValueAggregator, MinMaxSumCountAggregator,
            SumAggregator,
        },
        controllers::{self, PushController, PushControllerWorker},
        selectors::simple,
        ErrorHandler,
    },
};
use futures::Stream;
#[cfg(feature = "serialize")]
use serde::{Serialize, Serializer};
use std::fmt;
use std::io;
use std::sync::Mutex;
use std::time::{Duration, SystemTime};

/// TODO
pub fn stdout<S, SO, I, IS, ISI>(spawn: S, interval: I) -> StdoutExporterBuilder<io::Stdout, S, I>
where
    S: Fn(PushControllerWorker) -> SO,
    I: Fn(Duration) -> IS,
    IS: Stream<Item = ISI> + Send + 'static,
{
    StdoutExporterBuilder::<io::Stdout, S, I>::builder(spawn, interval)
}

/// TODO
#[derive(Debug)]
pub struct StdoutExporter<W> {
    writer: Mutex<W>,
    pretty_print: bool,
    do_not_print_time: bool,
    quantiles: Vec<f64>,
    label_encoder: Box<dyn labels::Encoder + Send + Sync>,
    formatter: Option<Formatter>,
}

/// TODO
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[derive(Default, Debug)]
pub struct ExporterBatch {
    #[cfg_attr(feature = "serialize", serde(skip_serializing_if = "Option::is_none"))]
    timestamp: Option<SystemTime>,
    updates: Vec<ExpoLine>,
}

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[derive(Default, Debug)]
struct ExpoLine {
    name: String,
    #[cfg_attr(feature = "serialize", serde(skip_serializing_if = "Option::is_none"))]
    min: Option<ExportNumeric>,
    #[cfg_attr(feature = "serialize", serde(skip_serializing_if = "Option::is_none"))]
    max: Option<ExportNumeric>,
    #[cfg_attr(feature = "serialize", serde(skip_serializing_if = "Option::is_none"))]
    sum: Option<ExportNumeric>,
    count: u64,
    #[cfg_attr(feature = "serialize", serde(skip_serializing_if = "Option::is_none"))]
    last_value: Option<ExportNumeric>,

    #[cfg_attr(feature = "serialize", serde(skip_serializing_if = "Option::is_none"))]
    quantiles: Option<Vec<ExporterQuantile>>,

    #[cfg_attr(feature = "serialize", serde(skip_serializing_if = "Option::is_none"))]
    timestamp: Option<SystemTime>,
}

/// TODO
pub struct ExportNumeric(Box<dyn fmt::Debug>);

impl fmt::Debug for ExportNumeric {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[cfg(feature = "serialize")]
impl Serialize for ExportNumeric {
    #[cfg(feature = "serialize")]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{:?}", self);
        serializer.serialize_str(&s)
    }
}

#[cfg_attr(feature = "serialize", derive(Serialize))]
#[derive(Debug)]
struct ExporterQuantile {
    q: f64,
    v: ExportNumeric,
}

impl<W> Exporter for StdoutExporter<W>
where
    W: fmt::Debug + io::Write,
{
    fn export(&self, checkpoint_set: &mut dyn CheckpointSet) -> Result<()> {
        let mut batch = ExporterBatch::default();
        if !self.do_not_print_time {
            batch.timestamp = Some(SystemTime::now());
        }
        checkpoint_set.try_for_each(&mut |record| {
            let desc = record.descriptor();
            let agg = record.aggregator();
            let kind = desc.number_kind();
            let encoded_resource = record.resource().encoded(self.label_encoder.as_ref());

            let mut expose = ExpoLine::default();

            if let Some(array) = agg.as_any().downcast_ref::<ArrayAggregator>() {
                expose.min = Some(ExportNumeric(array.min()?.to_debug(kind)));
                expose.max = Some(ExportNumeric(array.max()?.to_debug(kind)));
                expose.sum = Some(ExportNumeric(array.sum()?.to_debug(kind)));
                expose.count = array.count()?;

                let quantiles = self
                    .quantiles
                    .iter()
                    .map(|&q| {
                        Ok(ExporterQuantile {
                            q,
                            v: ExportNumeric(array.quantile(q)?.to_debug(kind)),
                        })
                    })
                    .collect::<Result<Vec<_>>>()?;
                expose.quantiles = Some(quantiles);
            }

            if let Some(last_value) = agg.as_any().downcast_ref::<LastValueAggregator>() {
                let (value, timestamp) = last_value.last_value()?;
                expose.last_value = Some(ExportNumeric(value.to_debug(kind)));

                if !self.do_not_print_time {
                    expose.timestamp = Some(timestamp);
                }
            }

            if let Some(histogram) = agg.as_any().downcast_ref::<HistogramAggregator>() {
                expose.sum = Some(ExportNumeric(histogram.sum()?.to_debug(kind)));
                expose.count = histogram.count()?;
                // TODO expose buckets
            }

            if let Some(mmsc) = agg.as_any().downcast_ref::<MinMaxSumCountAggregator>() {
                expose.min = Some(ExportNumeric(mmsc.min()?.to_debug(kind)));
                expose.max = Some(ExportNumeric(mmsc.max()?.to_debug(kind)));
                expose.sum = Some(ExportNumeric(mmsc.sum()?.to_debug(kind)));
                expose.count = mmsc.count()?;
            }

            if let Some(sum) = agg.as_any().downcast_ref::<SumAggregator>() {
                expose.sum = Some(ExportNumeric(sum.sum()?.to_debug(kind)));
            }

            let mut encoded_labels = String::new();
            let iter = record.labels().iter();
            if let (0, _) = iter.size_hint() {
                encoded_labels = record.labels().encoded(Some(self.label_encoder.as_ref()));
            }

            let mut sb = String::new();

            sb.push_str(desc.name());

            if !encoded_labels.is_empty() || !encoded_resource.is_empty() {
                sb.push_str("{");
                sb.push_str(&encoded_resource);
                if !encoded_labels.is_empty() && !encoded_resource.is_empty() {
                    sb.push_str(",");
                }
                sb.push_str(&encoded_labels);
                sb.push_str("}");
            }

            expose.name = sb;

            batch.updates.push(expose);
            Ok(())
        })?;

        self.writer.lock().map_err(From::from).and_then(|mut w| {
            let formatted = match &self.formatter {
                Some(formatter) => formatter.0(batch)?,
                None => format!("{:?}\n", batch),
            };
            w.write_all(formatted.as_bytes()).map_err(From::from)
        })
    }
}

/// TODO
pub struct Formatter(Box<dyn Fn(ExporterBatch) -> Result<String> + Send + Sync>);
impl fmt::Debug for Formatter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Formatter(closure)")
    }
}

/// TODO
#[derive(Debug)]
pub struct StdoutExporterBuilder<W, S, I> {
    spawn: S,
    interval: I,
    writer: Mutex<W>,
    pretty_print: bool,
    do_not_print_time: bool,
    quantiles: Option<Vec<f64>>,
    label_encoder: Option<Box<dyn labels::Encoder + Send + Sync>>,
    period: Option<Duration>,
    error_handler: Option<ErrorHandler>,
    formatter: Option<Formatter>,
}

impl<W, S, SO, I, IS, ISI> StdoutExporterBuilder<W, S, I>
where
    W: io::Write + fmt::Debug + Send + Sync + 'static,
    S: Fn(PushControllerWorker) -> SO,
    I: Fn(Duration) -> IS,
    IS: Stream<Item = ISI> + Send + 'static,
{
    fn builder(spawn: S, interval: I) -> StdoutExporterBuilder<io::Stdout, S, I> {
        StdoutExporterBuilder {
            spawn,
            interval,
            writer: Mutex::new(io::stdout()),
            pretty_print: false,
            do_not_print_time: false,
            quantiles: None,
            label_encoder: None,
            period: None,
            error_handler: None,
            formatter: None,
        }
    }
    /// TODO
    pub fn with_writer<W2: io::Write>(self, writer: W2) -> StdoutExporterBuilder<W2, S, I> {
        StdoutExporterBuilder {
            spawn: self.spawn,
            interval: self.interval,
            writer: Mutex::new(writer),
            pretty_print: self.pretty_print,
            do_not_print_time: self.do_not_print_time,
            quantiles: self.quantiles,
            label_encoder: self.label_encoder,
            period: self.period,
            error_handler: self.error_handler,
            formatter: self.formatter,
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
    pub fn with_period(self, period: Duration) -> Self {
        StdoutExporterBuilder {
            period: Some(period),
            ..self
        }
    }

    /// TODO
    pub fn with_error_handler<T>(self, handler: T) -> Self
    where
        T: Fn(MetricsError) + Send + Sync + 'static,
    {
        StdoutExporterBuilder {
            error_handler: Some(ErrorHandler::new(handler)),
            ..self
        }
    }

    /// TODO
    pub fn with_formatter<T>(self, formatter: T) -> Self
    where
        T: Fn(ExporterBatch) -> Result<String> + Send + Sync + 'static,
    {
        StdoutExporterBuilder {
            formatter: Some(Formatter(Box::new(formatter))),
            ..self
        }
    }

    /// TODO
    pub fn try_init(mut self) -> metrics::Result<PushController> {
        let period = self.period.take();
        let error_handler = self.error_handler.take();
        let (spawn, interval, exporter) = self.try_build()?;
        let mut push_builder =
            controllers::push(simple::Selector::Exact, exporter, spawn, interval)
                .with_stateful(true);
        if let Some(period) = period {
            push_builder = push_builder.with_period(period);
        }

        if let Some(error_handler) = error_handler {
            push_builder = push_builder.with_error_handler(error_handler);
        }
        let controller = push_builder.build();
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
                quantiles: self.quantiles.unwrap_or_else(|| vec![0.5, 0.9, 0.99]),
                label_encoder: self.label_encoder.unwrap_or_else(labels::default_encoder),
                formatter: self.formatter,
            },
        ))
    }
}
