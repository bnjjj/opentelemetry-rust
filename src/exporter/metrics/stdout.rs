//! Stdout Metrics Exporter
use crate::api::labels;
use crate::api::{
    metrics,
    metrics::{MetricsError, Result},
};
use crate::global;
use crate::sdk::{
    export::metrics::{CheckpointSet, Exporter},
    metrics::{
        aggregators::{
            ArrayAggregator, Count, DistributionAggregator, Max, Min, MinMaxSumCountAggregator,
            Quantile, Sum, SumAggregator,
        },
        controllers::{self, PushController, PushControllerWorker},
        selectors::simple,
        ErrorHandler,
    },
};
use futures::Stream;
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
}

#[derive(Default, Debug)]
struct ExpoBatch {
    timestamp: Option<SystemTime>,
    updates: Vec<ExpoLine>,
}

#[derive(Default, Debug)]
struct ExpoLine {
    name: String,
    min: Option<Box<dyn fmt::Debug>>,
    max: Option<Box<dyn fmt::Debug>>,
    sum: Option<Box<dyn fmt::Debug>>,
    count: u64,
    last_value: (),

    quantiles: Option<Vec<ExporterQuantile>>,

    timestamp: Option<SystemTime>,
}

#[derive(Debug)]
struct ExporterQuantile {
    q: f64,
    v: Box<dyn fmt::Debug>,
}

impl<W> Exporter for StdoutExporter<W>
where
    W: fmt::Debug + io::Write,
{
    fn export(&self, checkpoint_set: &mut dyn CheckpointSet) -> Result<()> {
        let mut batch = ExpoBatch::default();
        if !self.do_not_print_time {
            batch.timestamp = Some(SystemTime::now());
        }
        // checkpoint_set.try_for_each(process_record)?;
        checkpoint_set.try_for_each(&mut |record| {
            let desc = record.descriptor();
            let agg = record.aggregator();
            let kind = desc.number_kind();
            let encoded_resource = record.resource().encoded(self.label_encoder.as_ref());

            let mut expose = ExpoLine::default();

            if let Some(sum) = agg.as_any().downcast_ref::<SumAggregator>() {
                expose.sum = Some(sum.sum()?.to_debug(kind));
            }

            if let Some(array) = agg.as_any().downcast_ref::<ArrayAggregator>() {
                expose.min = Some(array.min()?.to_debug(kind));
                expose.max = Some(array.max()?.to_debug(kind));
                expose.sum = Some(array.sum()?.to_debug(kind));
                expose.count = array.count()?;

                let quantiles = self
                    .quantiles
                    .iter()
                    .map(|&q| {
                        Ok(ExporterQuantile {
                            q,
                            v: array.quantile(q)?.to_debug(kind),
                        })
                    })
                    .collect::<Result<Vec<_>>>()?;
                expose.quantiles = Some(quantiles);
            }

            if let Some(mmsc) = agg.as_any().downcast_ref::<MinMaxSumCountAggregator>() {
                expose.min = Some(mmsc.min()?.to_debug(kind));
                expose.max = Some(mmsc.max()?.to_debug(kind));
                expose.sum = Some(mmsc.sum()?.to_debug(kind));
                expose.count = mmsc.count()?;

                if let Some(_dist) = agg.as_any().downcast_ref::<DistributionAggregator>() {
                    if !self.quantiles.is_empty() {
                        todo!("NOT YET");
                        // summary := make([]expoQuantile, len(e.config.Quantiles))
                        // expose.Quantiles = summary
                        //
                        // for i, q := range e.config.Quantiles {
                        //         var vstr interface{}
                        //         value, err := dist.Quantile(q)
                        //         if err != nil {
                        //                 return err
                        //         }
                        //         vstr = value.AsInterface(kind)
                        //         summary[i] = expoQuantile{
                        //                 Q: q,
                        //                 V: vstr,
                        //         }
                        // }
                    }
                }
            } else {
                // } else if lv, ok := agg.(aggregator.LastValue); ok {
                // 	value, timestamp, err := lv.LastValue()
                // 	if err != nil {
                // 		return err
                // 	}
                // 	expose.LastValue = value.AsInterface(kind)
                //
                // 	if !e.config.DoNotPrintTime {
                // 		expose.Timestamp = &timestamp
                // 	}
            };

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
            w.write_all(format!("{:?}\n", batch).as_bytes())
                .map_err(From::from)
        })
        // let data = serde_json::to_value(batch)?;
        // if self.pretty_print {
        //     self.writer
        //         .write_all(format!("{:#}", data).as_bytes())
        //         .map_err(From::from)
        // } else {
        //     self.writer
        //         .write_all(data.to_string().as_bytes())
        //         .map_err(From::from)
        // }
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
            },
        ))
    }
}
