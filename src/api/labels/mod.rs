//! OpenTelemetry Labels
use crate::api::{KeyValue, Value};
use std::hash::{Hash, Hasher};
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

impl Set {
    /// TODO
    pub fn with_equivalent(equivalent: Distinct) -> Self {
        Set {
            equivalent,
            cached_encodings: Mutex::new([None, None, None]),
        }
    }
}

/// Distinct wraps a variable-size array of `kv.KeyValue`, constructed with keys
/// in sorted order. This can be used as a map key or for equality checking
/// between Sets.
#[derive(Debug, Default, PartialEq)]
pub struct Distinct(Vec<KeyValue>);

impl From<&[KeyValue]> for Distinct {
    fn from(kvs: &[KeyValue]) -> Self {
        let mut inner = kvs.to_vec();
        inner.sort_by(|a, b| a.key.cmp(&b.key));

        Distinct(inner)
    }
}

impl Eq for Distinct {}
impl Hash for Distinct {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for kv in self.0.iter() {
            kv.key.hash(state);

            match &kv.value {
                Value::Bool(b) => b.hash(state),
                Value::I64(i) => i.hash(state),
                Value::U64(u) => u.hash(state),
                Value::F64(f) => {
                    // FIXME: f64 does not impl hash, this impl may have incorrect outcomes.
                    f.to_bits().hash(state)
                }
                Value::String(s) => s.hash(state),
                Value::Bytes(b) => state.write(b),
            }
        }
    }
}
