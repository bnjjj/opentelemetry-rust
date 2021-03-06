//! SDK Configuration
//!
//! Configuration represents the global tracing configuration, overrides
//! can be set for the default OpenTelemetry limits and Sampler.
use crate::{api, sdk};
use std::sync::Arc;

/// Tracer configuration
#[derive(Debug)]
pub struct Config {
    /// The sampler that the sdk should use
    pub default_sampler: Box<dyn api::Sampler>,
    /// The id generator that the sdk should use
    pub id_generator: Box<dyn api::IdGenerator>,
    /// The max events that can be added to a `Span`.
    pub max_events_per_span: u32,
    /// The max attributes that can be added to a `Span`.
    pub max_attributes_per_span: u32,
    /// The max links that can be added to a `Span`.
    pub max_links_per_span: u32,
    /// Contains attributes representing an entity that produces telemetry.
    pub resource: Arc<sdk::Resource>,
}

impl Default for Config {
    /// Create default global sdk configuration.
    fn default() -> Self {
        Config {
            default_sampler: Box::new(sdk::Sampler::AlwaysOn),
            id_generator: Box::new(sdk::IdGenerator::default()),
            max_events_per_span: 128,
            max_attributes_per_span: 32,
            max_links_per_span: 32,
            resource: Arc::new(sdk::Resource::default()),
        }
    }
}
