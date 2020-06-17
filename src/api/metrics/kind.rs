/// Kinds of OpenTelemetry metric inetruments
///
/// | **Name** | Instrument kind | Function(argument) | Default aggregation | Notes |
/// | ----------------------- | ----- | --------- | ------------- | --- |
/// | **ValueRecorder**       | Synchronous  | Record(value) | MinMaxSumCount  | Per-request, any non-additive measurement |
/// | **ValueObserver**       | Asynchronous | Observe(value) | MinMaxSumCount  | Per-interval, any non-additive measurement |
/// | **Counter**             | Synchronous additive monotonic | Add(increment) | Sum | Per-request, part of a monotonic sum |
/// | **UpDownCounter**       | Synchronous additive | Add(increment) | Sum | Per-request, part of a non-monotonic sum |
/// | **SumObserver**         | Asynchronous additive monotonic | Observe(sum) | Sum | Per-interval, reporting a monotonic sum |
/// | **UpDownSumObserver**   | Asynchronous additive | Observe(sum) | Sum | Per-interval, reporting a non-monotonic sum |
#[derive(Clone, Debug, PartialEq, Hash)]
pub enum InstrumentKind {
    /// A synchronous per-request recorder of non-additive measurements.
    ValueRecorder,
    /// An asynchronous per-interval recorder of non-additive measurements.
    ValueObserver,
    /// A synchronous per-request part of a monotonic sum.
    Counter,
    /// A synchronous per-request part of a non-monotonic sum.
    UpDownCounter,
    /// An asynchronous per-interval recorder of a monotonic sum.
    SumObserver,
    /// An asynchronous per-interval recorder of a non-monotonic sum.
    UpDownSumObserver,
}
