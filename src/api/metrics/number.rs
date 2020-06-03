use crate::api::metrics::NumberKind;
use std::sync::atomic::{AtomicU64, Ordering};

/// Number represents either an integral or a floating point value. It
/// needs to be accompanied with a source of NumberKind that describes
/// the actual type of the value stored within Number.
#[derive(Debug, Default)]
pub struct Number(AtomicU64);

impl Number {
    /// TODO
    pub fn add(&self, number_kind: &NumberKind, other: Number) {
        let current = self.0.load(Ordering::Acquire);
        let other = other.0.load(Ordering::Acquire);
        match number_kind {
            NumberKind::F64 => loop {
                let new = f64::from_bits(current) + f64::from_bits(other);
                let swapped = self
                    .0
                    .compare_and_swap(current, new.to_bits(), Ordering::Release);
                if swapped == current {
                    return;
                }
            },
            NumberKind::U64 => loop {
                let new = current + other;
                let swapped = self.0.compare_and_swap(current, new, Ordering::Release);
                if swapped == current {
                    return;
                }
            },
        }
    }

    /// TODO
    pub fn is_nan(&self) -> bool {
        let current = self.0.load(Ordering::Acquire);
        f64::from_bits(current).is_nan()
    }

    /// TODO
    pub fn is_negative(&self, number_kind: &NumberKind) -> bool {
        match number_kind {
            NumberKind::U64 => true,
            NumberKind::F64 => {
                let current = self.0.load(Ordering::Acquire);
                f64::from_bits(current).is_sign_positive()
            }
        }
    }
}

impl From<f64> for Number {
    fn from(f: f64) -> Self {
        Number(AtomicU64::new(f.to_bits()))
    }
}

impl From<i64> for Number {
    fn from(i: i64) -> Self {
        Number(AtomicU64::new(i as u64))
    }
}

impl From<u64> for Number {
    fn from(u: u64) -> Self {
        Number(AtomicU64::new(u))
    }
}
