use crate::api::KeyValue;
use std::fmt;
use std::sync::atomic::{AtomicUsize, Ordering};

static ENCODER_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

/// TODO
pub trait Encoder: fmt::Debug {
    /// Encode returns the serialized encoding of the label
    /// set using its Iterator.  This result may be cached
    /// by a label.Set.
    fn encode(&self, labels: &mut dyn Iterator<Item = &KeyValue>) -> String;

    /// ID returns a value that is unique for each class of
    /// label encoder.  Label encoders allocate these using
    /// `NewEncoderID`.
    fn id(&self) -> EncoderId;
}

/// EncoderID is used to identify distinct Encoder
/// implementations, for caching encoded results.
#[derive(Debug)]
pub struct EncoderId(usize);

impl EncoderId {
    /// TODO
    pub fn is_valid(&self) -> bool {
        self.0 != 0
    }
}

/// TODO
#[derive(Debug)]
pub struct DefaultLabelEncoder;

impl Encoder for DefaultLabelEncoder {
    fn encode(&self, labels: &mut dyn Iterator<Item = &KeyValue>) -> String {
        // TODO fix horrible perf
        labels
            .enumerate()
            .fold(String::new(), |mut acc, (idx, kv)| {
                if idx > 0 {
                    acc.push_str(",")
                }
                acc.push_str(kv.key.as_str());
                acc.push_str("=");
                acc.push_str(String::from(&kv.value).as_str());
                acc
            })
    }

    fn id(&self) -> EncoderId {
        new_encoder_id()
    }
}

/// TODO
pub fn default_encoder() -> Box<dyn Encoder + Send + Sync> {
    Box::new(DefaultLabelEncoder)
}

/// TODO
pub fn new_encoder_id() -> EncoderId {
    let old_encoder_id = ENCODER_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
    EncoderId(old_encoder_id + 1)
}
