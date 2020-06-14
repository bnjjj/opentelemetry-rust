#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate prometheus;

use hyper::{
    header::CONTENT_TYPE,
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};
use opentelemetry::api::{
    metrics::{BoundCounter, BoundValueRecorder},
    KeyValue,
};
use opentelemetry::global;
use prometheus::{Counter, Encoder, Gauge, HistogramVec, TextEncoder};
use std::convert::Infallible;
use std::sync::Arc;

lazy_static! {
    static ref HANDLER_ALL: [KeyValue; 1] = [KeyValue::new("handler", "all")];
    static ref HTTP_COUNTER: Counter = register_counter!(opts!(
        "example_http_requests_total",
        "Total number of HTTP requests made.",
        labels! {"handler" => "all",}
    ))
    .unwrap();
    static ref HTTP_BODY_GAUGE: Gauge = register_gauge!(opts!(
        "example_http_response_size_bytes",
        "The HTTP response sizes in bytes.",
        labels! {"handler" => "all",}
    ))
    .unwrap();
    static ref HTTP_REQ_HISTOGRAM: HistogramVec = register_histogram_vec!(
        "example_http_request_duration_seconds",
        "The HTTP request latencies in seconds.",
        &["handler"]
    )
    .unwrap();
}

async fn serve_req(
    _req: Request<Body>,
    state: Arc<AppState>,
) -> Result<Response<Body>, hyper::Error> {
    let encoder = TextEncoder::new();

    // HTTP_COUNTER.inc();
    state.http_counter.add(1);
    let timer = HTTP_REQ_HISTOGRAM.with_label_values(&["all"]).start_timer();

    let metric_families = prometheus::gather();
    let mut buffer = vec![];
    encoder.encode(&metric_families, &mut buffer).unwrap();
    HTTP_BODY_GAUGE.set(buffer.len() as f64);

    let response = Response::builder()
        .status(200)
        .header(CONTENT_TYPE, encoder.format_type())
        .body(Body::from(buffer))
        .unwrap();

    timer.observe_duration();

    Ok(response)
}
fn init_meter() {
    opentelemetry_prometheus::exporter().init()
}

struct AppState {
    http_counter: BoundCounter<'static, u64>,
    http_body_gauge: BoundValueRecorder<'static, u64>,
    // http_req_histogram: SyncInstrument<f64>,
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    init_meter();

    let meter = global::meter("ex.com/hyper");
    let state = Arc::new(AppState {
        http_counter: meter
            .u64_counter("example.http_requests_total")
            .with_description("Total number of HTTP requests made.")
            .init()
            .bind(HANDLER_ALL.as_ref()),
    });

    // For every connection, we must make a `Service` to handle all
    // incoming HTTP requests on said connection.
    let make_svc = make_service_fn(move |_conn| {
        let state = state.clone();
        // This is the `Service` that will handle the connection.
        // `service_fn` is a helper to convert a function that
        // returns a Response into a `Service`.
        async move { Ok::<_, Infallible>(service_fn(move |req| serve_req(req, state.clone()))) }
    });

    let addr = ([127, 0, 0, 1], 3000).into();

    let server = Server::bind(&addr).serve(make_svc);

    println!("Listening on http://{}", addr);

    server.await?;

    Ok(())
}

// use hyper::{
//     header::CONTENT_TYPE,
//     service::{make_service_fn, service_fn},
//     Body, Request, Response, Server,
// };
// use opentelemetry::api::{metrics::Result, KeyValue};
// use opentelemetry::sdk::metrics::{BoundInstrument, PullController};
// use prometheus::{Counter, Encoder, Gauge, HistogramVec, TextEncoder};
//
// lazy_static! {
//     static ref HANDLER_ALL: KeyValue = KeyValue::new("handler", "all");
//     // static ref HTTP_COUNTER: KeyValue = register_counter!(opts!(
//     //     "example_http_requests_total",
//     //     "Total number of HTTP requests made.",
//     //     labels! {"handler" => "all",}
//     // ))
//     // .unwrap();
//     // static ref HTTP_BODY_GAUGE: Gauge = register_gauge!(opts!(
//     //     "example_http_response_size_bytes",
//     //     "The HTTP response sizes in bytes.",
//     //     labels! {"handler" => "all",}
//     // ))
//     // .unwrap();
//     // static ref HTTP_REQ_HISTOGRAM: HistogramVec = register_histogram_vec!(
//     //     "example_http_request_duration_seconds",
//     //     "The HTTP request latencies in seconds.",
//     //     &["handler"]
//     // )
//     // .unwrap();
// }
//
// async fn serve_req(_req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
//     let encoder = TextEncoder::new();
//
//     // HTTP_COUNTER.inc();
//     // let timer = HTTP_REQ_HISTOGRAM.with_label_values(&["all"]).start_timer();
//     //
//     let metric_families = prometheus::gather();
//     let mut buffer = vec![];
//     encoder.encode(&metric_families, &mut buffer).unwrap();
//     // HTTP_BODY_GAUGE.set(buffer.len() as f64);
//
//     let response = Response::builder()
//         .status(200)
//         .header(CONTENT_TYPE, encoder.format_type())
//         .body(Body::from(buffer))
//         .unwrap();
//
//     // timer.observe_duration();
//
//     Ok(response)
// }
//
// // fn init_meter() -> metrics::Result<PullController> {
// //     opentelemetry_prometheus::exporter().try_init()
// // }
//
// // struct MyInstruments {
// //     http_counter: BoundInstrument<u64>,
// //     http_body_gauge: BoundInstrument<u64>,
// //     http_req_histogram: BoundInstrument<f64>,
// // }
//
// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
//     // let _started = init_meter();
//     let addr = ([127, 0, 0, 1], 9898).into();
//     println!("Listening on http://{}", addr);
//
//     let serve_future = Server::bind(&addr).serve(make_service_fn(|_| async {
//         Ok::<_, hyper::Error>(service_fn(serve_req))
//     }));
//
//     if let Err(err) = serve_future.await {
//         eprintln!("server error: {}", err);
//     }
//
//     Ok(())
// }
