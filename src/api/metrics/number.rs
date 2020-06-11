use std::cmp;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};

/// Number represents either an integral or a floating point value. It
/// needs to be accompanied with a source of NumberKind that describes
/// the actual type of the value stored within Number.
#[derive(Debug, Default)]
pub struct Number(AtomicU64);

impl Number {
    /// TODO
    pub fn assign(&self, number_kind: &NumberKind, other: &Number) {
        let current = self.0.load(Ordering::Acquire);
        let other = other.0.load(Ordering::Acquire);
        match number_kind {
            NumberKind::F64 => loop {
                let new = f64::from_bits(other);
                let swapped = self
                    .0
                    .compare_and_swap(current, new.to_bits(), Ordering::Release);
                if swapped == current {
                    return;
                }
            },
            NumberKind::U64 => loop {
                let new = other;
                let swapped = self.0.compare_and_swap(current, new, Ordering::Release);
                if swapped == current {
                    return;
                }
            },
        }
    }

    /// TODO
    pub fn add(&self, number_kind: &NumberKind, other: &Number) {
        match number_kind {
            NumberKind::F64 => loop {
                let current = self.0.load(Ordering::Acquire);
                let other = other.0.load(Ordering::Acquire);
                let new = f64::from_bits(current) + f64::from_bits(other);
                let swapped = self
                    .0
                    .compare_and_swap(current, new.to_bits(), Ordering::Release);
                if swapped == current {
                    return;
                }
            },
            NumberKind::U64 => loop {
                let current = self.0.load(Ordering::Acquire);
                let other = other.0.load(Ordering::Acquire);
                let new = current + other;
                let swapped = self.0.compare_and_swap(current, new, Ordering::Release);
                if swapped == current {
                    return;
                }
            },
        }
    }

    /// TODO
    pub fn to_u64(&self) -> u64 {
        self.0.load(Ordering::SeqCst)
    }

    /// TODO
    pub fn to_f64(&self, number_kind: &NumberKind) -> f64 {
        let current = self.0.load(Ordering::SeqCst);

        match number_kind {
            NumberKind::U64 => current as f64,
            NumberKind::F64 => f64::from_bits(current),
        }
    }

    /// TODO
    pub fn partial_cmp(&self, number_kind: &NumberKind, other: &Number) -> Option<cmp::Ordering> {
        let current = self.0.load(Ordering::SeqCst);
        let other = other.0.load(Ordering::SeqCst);
        match number_kind {
            NumberKind::F64 => {
                let current = f64::from_bits(current);
                let other = f64::from_bits(other);
                current.partial_cmp(&other)
            }
            NumberKind::U64 => current.partial_cmp(&other),
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

    /// TODO
    pub fn to_debug(&self, kind: &NumberKind) -> Box<dyn fmt::Debug> {
        let current = self.0.load(Ordering::SeqCst);
        match kind {
            NumberKind::U64 => Box::new(current),
            NumberKind::F64 => Box::new(f64::from_bits(current)),
        }
    }
}

impl Clone for Number {
    fn clone(&self) -> Self {
        self.0.load(Ordering::SeqCst).into()
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

/// TODO
#[derive(Clone, Debug, PartialEq, Hash)]
pub enum NumberKind {
    /// TODO
    F64,
    /// TODO
    U64,
}

impl NumberKind {
    /// TODO
    pub fn zero(&self) -> Number {
        Number::default()
    }

    /// TODO
    pub fn max(&self) -> Number {
        match self {
            NumberKind::U64 => u64::MAX.into(),
            NumberKind::F64 => f64::MAX.into(),
        }
    }

    /// TODO
    pub fn min(&self) -> Number {
        match self {
            NumberKind::U64 => u64::MIN.into(),
            NumberKind::F64 => f64::MIN.into(),
        }
    }
}
