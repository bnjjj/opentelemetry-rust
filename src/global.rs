//! # OpenTelemetry Global API
//!
//! The global API **provides applications access to their configured
//! [`Provider`] instance from anywhere in the codebase**. This allows
//! applications to be less coupled to the specific Open Telemetry SDK as
//! well as not manually pass references to each part of the code that needs
//! to create [`Span`]s. Additionally, **3rd party middleware** or **library code**
//! can be written against this generic API and not constrain users to a
//! specific implementation choice.
//!
//! ## Usage
//!
//! ```rust
//! use opentelemetry::api::{Provider, Tracer};
//! use opentelemetry::global;
//!
//! fn init_tracer() {
//!     let provider = opentelemetry::api::NoopProvider {};
//!
//!     // Configure the global `Provider` singleton when your app starts
//!     // (there is a no-op default if this is not set by your application)
//!     global::set_provider(provider);
//! }
//!
//! fn do_something_tracked() {
//!     // Then you can use the global provider to create a tracer via `tracer`.
//!     let _span = global::tracer("my-component").start("span-name");
//!
//!     // Or access the configured provider via `trace_provider`.
//!     let provider = global::trace_provider();
//!     let _tracer_a = provider.get_tracer("my-component-a");
//!     let _tracer_b = provider.get_tracer("my-component-b");
//! }
//!
//! // in main or other app start
//! init_tracer();
//! do_something_tracked();
//! ```
//!
//! ## Implementation
//!
//! This module provides types for working with the Open Telemetry API in an
//! abstract implementation-agnostic way through the use of [trait objects].
//! There is a **performance penalty** due to global synchronization as well
//! as heap allocation and dynamic dispatch (e.g. `Box<DynSpan>` vs
//! `sdk::Span`), but for many applications this overhead is likely either
//! insignificant or unavoidable as it is in the case of 3rd party integrations
//! that do not know the span type at compile time.
//!
//! ### Generic interface
//!
//! The generic interface is provided by the [`GlobalProvider`] struct which
//! can be accessed anywhere via [`trace_provider`] and allows applications to
//! use the [`BoxedTracer`] and [`BoxedSpan`] instances that implement
//! [`Tracer`] and [`Span`]. They wrap a boxed dyn [`GenericProvider`],
//! [`GenericTracer`], and [`Span`] respectively allowing the underlying
//! implementation to be set at runtime.
//!
//! [`Provider`]: ../api/trace/provider/trait.Provider.html
//! [`Tracer`]: ../api/trace/tracer/trait.Tracer.html
//! [`Span`]: ../api/trace/span/trait.Span.html
//! [`GenericProvider`]: trait.GenericProvider.html
//! [`GenericTracer`]: trait.GenericTracer.html
//! [`GlobalProvider`]: struct.GlobalProvider.html
//! [`BoxedTracer`]: struct.BoxedTracer.html
//! [`BoxedSpan`]: struct.BoxedSpan.html
//! [`trace_provider`]: fn.trace_provider.html
//! [trait objects]: https://doc.rust-lang.org/reference/types/trait-object.html#trait-objects
use crate::{api, api::Provider};
use std::fmt;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;

/// Wraps the [`BoxedTracer`]'s [`Span`] so it can be used generically by
/// applications without knowing the underlying type.
///
/// [`BoxedTracer`]: struct.BoxedTracer.html
/// [`Span`]: ../api/trace/span/trait.Span.html
#[derive(Debug)]
pub struct BoxedSpan(Box<DynSpan>);
type DynSpan = dyn api::Span + Send + Sync;

impl api::Span for BoxedSpan {
    /// Records events at a specific time in the context of a given `Span`.
    ///
    /// Note that the OpenTelemetry project documents certain ["standard event names and
    /// keys"](https://github.com/open-telemetry/opentelemetry-specification/tree/v0.5.0/specification/trace/semantic_conventions/README.md)
    /// which have prescribed semantic meanings.
    fn add_event_with_timestamp(
        &self,
        name: String,
        timestamp: SystemTime,
        attributes: Vec<api::KeyValue>,
    ) {
        self.0.add_event_with_timestamp(name, timestamp, attributes)
    }

    /// Returns the `SpanContext` for the given `Span`.
    fn span_context(&self) -> api::SpanContext {
        self.0.span_context()
    }

    /// Returns true if this `Span` is recording information like events with the `add_event`
    /// operation, attributes using `set_attributes`, status with `set_status`, etc.
    fn is_recording(&self) -> bool {
        self.0.is_recording()
    }

    /// Sets a single `Attribute` where the attribute properties are passed as arguments.
    ///
    /// Note that the OpenTelemetry project documents certain ["standard
    /// attributes"](https://github.com/open-telemetry/opentelemetry-specification/tree/v0.5.0/specification/trace/semantic_conventions/README.md)
    /// that have prescribed semantic meanings.
    fn set_attribute(&self, attribute: api::KeyValue) {
        self.0.set_attribute(attribute)
    }

    /// Sets the status of the `Span`. If used, this will override the default `Span`
    /// status, which is `OK`.
    fn set_status(&self, code: api::StatusCode, message: String) {
        self.0.set_status(code, message)
    }

    /// Updates the `Span`'s name.
    fn update_name(&self, new_name: String) {
        self.0.update_name(new_name)
    }

    /// Finishes the span.
    fn end(&self) {
        self.0.end()
    }
}

/// Wraps the [`GlobalProvider`]'s [`Tracer`] so it can be used generically by
/// applications without knowing the underlying type.
///
/// [`GlobalProvider`]: struct.GlobalProvider.html
/// [`Tracer`]: ../api/trace/tracer/trait.Tracer.html
#[derive(Debug)]
pub struct BoxedTracer(Box<dyn GenericTracer + Send + Sync>);

impl api::Tracer for BoxedTracer {
    /// Global tracer uses `BoxedSpan`s so that it can be a global singleton,
    /// which is not possible if it takes generic type parameters.
    type Span = BoxedSpan;

    /// Returns a span with an inactive `SpanContext`. Used by functions that
    /// need to return a default span like `get_active_span` if no span is present.
    fn invalid(&self) -> Self::Span {
        BoxedSpan(self.0.invalid_boxed())
    }

    /// Starts a new `Span`.
    ///
    /// Each span has zero or one parent spans and zero or more child spans, which
    /// represent causally related operations. A tree of related spans comprises a
    /// trace. A span is said to be a _root span_ if it does not have a parent. Each
    /// trace includes a single root span, which is the shared ancestor of all other
    /// spans in the trace.
    fn start_from_context(&self, name: &str, cx: &api::Context) -> Self::Span {
        BoxedSpan(self.0.start_with_context_boxed(name, cx))
    }

    /// Creates a span builder
    ///
    /// An ergonomic way for attributes to be configured before the `Span` is started.
    fn span_builder(&self, name: &str) -> api::SpanBuilder {
        api::SpanBuilder::from_name(name.to_string())
    }

    /// Create a span from a `SpanBuilder`
    fn build_with_context(&self, builder: api::SpanBuilder, cx: &api::Context) -> Self::Span {
        BoxedSpan(self.0.build_with_context_boxed(builder, cx))
    }
}

/// Allows a specific [`Tracer`] to be used generically by [`BoxedTracer`]
/// instances by mirroring the interface and boxing the return types.
///
/// [`Tracer`]: ../api/trace/tracer/trait.Tracer.html
/// [`BoxedTracer`]: struct.BoxedTracer.html
pub trait GenericTracer: fmt::Debug + 'static {
    /// Create a new invalid span for use in cases where there are no active spans.
    fn invalid_boxed(&self) -> Box<DynSpan>;

    /// Returns a trait object so the underlying implementation can be swapped
    /// out at runtime.
    fn start_with_context_boxed(&self, name: &str, cx: &api::Context) -> Box<DynSpan>;

    /// Returns a trait object so the underlying implementation can be swapped
    /// out at runtime.
    fn build_with_context_boxed(
        &self,
        builder: api::SpanBuilder,
        cx: &api::Context,
    ) -> Box<DynSpan>;
}

impl<S, T> GenericTracer for T
where
    S: api::Span + Send + Sync,
    T: api::Tracer<Span = S>,
{
    /// Create a new invalid span for use in cases where there are no active spans.
    fn invalid_boxed(&self) -> Box<DynSpan> {
        Box::new(self.invalid())
    }

    /// Returns a trait object so the underlying implementation can be swapped
    /// out at runtime.
    fn start_with_context_boxed(&self, name: &str, cx: &api::Context) -> Box<DynSpan> {
        Box::new(self.start_from_context(name, cx))
    }

    /// Returns a trait object so the underlying implementation can be swapped
    /// out at runtime.
    fn build_with_context_boxed(
        &self,
        builder: api::SpanBuilder,
        cx: &api::Context,
    ) -> Box<DynSpan> {
        Box::new(self.build_with_context(builder, cx))
    }
}

/// Allows a specific [`Provider`] to be used generically by the
/// [`GlobalProvider`] by mirroring the interface and boxing the return types.
///
/// [`Provider`]: ../api/trace/provider/trait.Provider.html
/// [`GlobalProvider`]: struct.GlobalProvider.html
pub trait GenericProvider: fmt::Debug + 'static {
    /// Creates a named tracer instance that is a trait object through the underlying `Provider`.
    fn get_tracer_boxed(&self, name: &'static str) -> Box<dyn GenericTracer + Send + Sync>;
}

impl<S, T, P> GenericProvider for P
where
    S: api::Span + Send + Sync,
    T: api::Tracer<Span = S> + Send + Sync,
    P: api::Provider<Tracer = T>,
{
    /// Return a boxed generic tracer
    fn get_tracer_boxed(&self, name: &'static str) -> Box<dyn GenericTracer + Send + Sync> {
        Box::new(self.get_tracer(name))
    }
}

/// Represents the globally configured [`Provider`] instance for this
/// application. This allows generic tracing through the returned
/// [`BoxedTracer`] instances.
///
/// [`Provider`]: ../api/trace/provider/trait.Provider.html
/// [`BoxedTracer`]: struct.BoxedTracer.html
#[derive(Clone, Debug)]
pub struct GlobalProvider {
    provider: Arc<dyn GenericProvider + Send + Sync>,
}

impl GlobalProvider {
    /// Create a new GlobalProvider instance from a struct that implements `Provider`.
    fn new<P, T, S>(provider: P) -> Self
    where
        S: api::Span + Send + Sync,
        T: api::Tracer<Span = S> + Send + Sync,
        P: api::Provider<Tracer = T> + Send + Sync,
    {
        GlobalProvider {
            provider: Arc::new(provider),
        }
    }
}

impl api::Provider for GlobalProvider {
    type Tracer = BoxedTracer;

    /// Find or create a named tracer using the global provider.
    fn get_tracer(&self, name: &'static str) -> Self::Tracer {
        BoxedTracer(self.provider.get_tracer_boxed(name))
    }
}

lazy_static::lazy_static! {
    /// The global `Tracer` singleton.
    static ref GLOBAL_TRACER_PROVIDER: RwLock<GlobalProvider> = RwLock::new(GlobalProvider::new(api::NoopProvider {}));
    /// The current global `HttpTextFormat` propagator.
    static ref GLOBAL_HTTP_TEXT_PROPAGATOR: RwLock<Box<dyn api::HttpTextFormat + Send + Sync>> = RwLock::new(Box::new(api::HttpTextCompositePropagator::new(vec![Box::new(api::TraceContextPropagator::new()), Box::new(api::CorrelationContextPropagator::new())])));
    /// The global default `HttpTextFormat` propagator.
    static ref DEFAULT_HTTP_TEXT_PROPAGATOR: api::HttpTextCompositePropagator = api::HttpTextCompositePropagator::new(vec![Box::new(api::TraceContextPropagator::new()), Box::new(api::CorrelationContextPropagator::new())]);
}

/// Returns an instance of the currently configured global [`Provider`] through
/// [`GlobalProvider`].
///
/// [`Provider`]: ../api/trace/provider/trait.Provider.html
/// [`GlobalProvider`]: struct.GlobalProvider.html
pub fn trace_provider() -> GlobalProvider {
    GLOBAL_TRACER_PROVIDER
        .read()
        .expect("GLOBAL_TRACER_PROVIDER RwLock poisoned")
        .clone()
}

/// Creates a named instance of [`Tracer`] via the configured [`GlobalProvider`].
///
/// If the name is an empty string, the provider will use a default name.
///
/// This is a more convenient way of expressing `global::trace_provider().get_tracer(name)`.
///
/// [`Tracer`]: ../api/trace/tracer/trait.Tracer.html
/// [`GlobalProvider`]: struct.GlobalProvider.html
pub fn tracer(name: &'static str) -> BoxedTracer {
    trace_provider().get_tracer(name)
}

/// Sets the given [`Provider`] instance as the current global provider.
///
/// [`Provider`]: ../api/trace/provider/trait.Provider.html
pub fn set_provider<P, T, S>(new_provider: P)
where
    S: api::Span + Send + Sync,
    T: api::Tracer<Span = S> + Send + Sync,
    P: api::Provider<Tracer = T> + Send + Sync,
{
    let mut global_provider = GLOBAL_TRACER_PROVIDER
        .write()
        .expect("GLOBAL_TRACER_PROVIDER RwLock poisoned");
    *global_provider = GlobalProvider::new(new_provider);
}

/// Sets the given [`HttpTextFormat`] propagator as the current global propagator.
///
/// [`HttpTextFormat`]: ../api/context/propagation/trait.HttpTextFormat.html
///
/// # Examples
///
/// ```
/// use opentelemetry::{api, global};
///
/// // create your http text propagator
/// let propagator = api::TraceContextPropagator::new();
///
/// // assign it as the global propagator
/// global::set_http_text_propagator(propagator);
/// ```
pub fn set_http_text_propagator<P: api::HttpTextFormat + Send + Sync + 'static>(propagator: P) {
    let _lock = GLOBAL_HTTP_TEXT_PROPAGATOR
        .write()
        .map(|mut global_propagator| *global_propagator = Box::new(propagator));
}

/// Executes a closure with a reference to the current global [`HttpTextFormat`] propagator.
///
/// [`HttpTextFormat`]: ../api/context/propagation/trait.HttpTextFormat.html
///
/// # Examples
///
/// ```
/// use opentelemetry::{api, api::HttpTextFormat, global};
/// use std::collections::HashMap;
///
/// let example_carrier = HashMap::new();
///
/// // create your http text propagator
/// let tc_propagator = api::TraceContextPropagator::new();
/// global::set_http_text_propagator(tc_propagator);
///
/// // use the global http text propagator to extract contexts
/// let _cx = global::get_http_text_propagator(|propagator| propagator.extract(&example_carrier));
/// ```
pub fn get_http_text_propagator<T, F>(mut f: F) -> T
where
    F: FnMut(&dyn api::HttpTextFormat) -> T,
{
    GLOBAL_HTTP_TEXT_PROPAGATOR
        .read()
        .map(|propagator| f(&**propagator))
        .unwrap_or_else(|_| f(&*DEFAULT_HTTP_TEXT_PROPAGATOR as &dyn api::HttpTextFormat))
}

/// Returns [`NoopMeter`] for now
///
/// [`NoopMeter`]: ../api/trace/noop/struct.NoopMeter.html
pub fn global_meter() -> crate::api::NoopMeter {
    crate::api::NoopMeter {}
}
