//! OpenTelemetry Labels
use crate::api::KeyValue;
use std::sync::Mutex;

const MAX_CONCURRENT_ENCODERS: usize = 3;

mod encoder;
pub use encoder::{default_encoder, new_encoder_id, DefaultLabelEncoder, Encoder, EncoderId};

/// Set is the representation for a distinct label set.  It manages an immutable
/// set of labels, with an internal cache for storing label encodings.
///
/// This type supports the `Equivalent` method of comparison using values of
/// type `Distinct`.
///
/// This type is used to implement:
/// 1. Metric labels
/// 2. Resource sets
/// 3. Correlation map (TODO)
#[derive(Debug, Default)]
pub struct Set {
    equivalent: Distinct,
    cached_encodings: Mutex<[Option<(EncoderId, String)>; MAX_CONCURRENT_ENCODERS]>,
}

/// Distinct wraps a variable-size array of `kv.KeyValue`, constructed with keys
/// in sorted order. This can be used as a map key or for equality checking
/// between Sets.
#[derive(Debug, Default, PartialEq)]
pub struct Distinct(Vec<KeyValue>);
