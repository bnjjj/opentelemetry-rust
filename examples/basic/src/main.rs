use opentelemetry::api::metrics::{self, F64ObserverResult};
use opentelemetry::api::{Context, CorrelationContextExt, Key, KeyValue, TraceContextExt, Tracer};
use opentelemetry::exporter;
use opentelemetry::sdk::metrics::PushController;
use opentelemetry::{global, sdk};

fn init_tracer() -> thrift::Result<()> {
    let exporter = opentelemetry_jaeger::Exporter::builder()
        .with_agent_endpoint("127.0.0.1:6831".parse().unwrap())
        .with_process(opentelemetry_jaeger::Process {
            service_name: "trace-demo".to_string(),
            tags: vec![
                Key::new("exporter").string("jaeger"),
                Key::new("float").f64(312.23),
            ],
        })
        .init()?;

    // For the demonstration, use `Sampler::Always` sampler to sample all traces. In a production
    // application, use `Sampler::Parent` or `Sampler::Probability` with a desired probability.
    let provider = sdk::Provider::builder()
        .with_simple_exporter(exporter)
        .with_config(sdk::Config {
            default_sampler: Box::new(sdk::Sampler::Always),
            ..Default::default()
        })
        .build();
    global::set_provider(provider);

    Ok(())
}

fn init_meter() -> metrics::Result<PushController> {
    exporter::metrics::stdout(tokio::spawn, tokio::time::interval)
        .with_quantiles(vec![0.5, 0.9, 0.99])
        .with_pretty_print(false)
        .try_init()
}

lazy_static::lazy_static! {
    static ref FOO_KEY: Key = Key::new("ex.com/foo");
    static ref BAR_KEY: Key = Key::new("ex.com/bar");
    static ref LEMONS_KEY: Key = Key::new("ex.com/lemons");
    static ref ANOTHER_KEY: Key = Key::new("ex.com/another");
    static ref COMMON_LABELS: [KeyValue; 4] = [
        LEMONS_KEY.i64(10),
        KeyValue::new("A", "1"),
        KeyValue::new("B", "2"),
        KeyValue::new("C", "3"),
    ];
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracer()?;
    let metrics_controller = init_meter()?;
    let _start = metrics_controller.start();

    let meter = global::meter("ex.com/basic");

    let one_metric_callback = |res: F64ObserverResult| res.observe(1.0, COMMON_LABELS.as_ref());
    let _ = meter
        .f64_value_observer("ex.com.one", one_metric_callback)
        .with_description("A ValueObserver set to 1.0")
        .init();

    let value_recorder_two = meter.f64_value_recorder("ex.com.two").init();

    let _correlations =
        Context::current_with_correlations(vec![FOO_KEY.string("foo1"), BAR_KEY.string("bar1")])
            .attach();

    let value_recorder = value_recorder_two.bind(COMMON_LABELS.as_ref());

    global::tracer("component-main").in_span("operation", move |cx| {
        let span = cx.span();
        span.add_event(
            "Nice operation!".to_string(),
            vec![Key::new("bogons").i64(100)],
        );
        span.set_attribute(ANOTHER_KEY.string("yes"));

        meter.record_batch_with_context(
            // Note: call-site variables added as context Entries:
            &Context::current_with_correlations(vec![ANOTHER_KEY.string("xyz")]),
            COMMON_LABELS.as_ref(),
            vec![value_recorder_two.measurement(2.0)],
        );

        global::tracer("component-bar").in_span("Sub operation...", move |cx| {
            let span = cx.span();
            span.set_attribute(LEMONS_KEY.string("five"));

            span.add_event("Sub span event".to_string(), vec![]);

            value_recorder.record(1.3);
        });
    });

    Ok(())
}
