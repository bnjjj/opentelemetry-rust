/// Number represents either an integral or a floating point value. It
/// needs to be accompanied with a source of NumberKind that describes
/// the actual type of the value stored within Number.
#[derive(Debug)]
pub struct Number(u64);

impl From<f64> for Number {
    fn from(f: f64) -> Self {
        Number(f.to_bits())
    }
}

impl From<i64> for Number {
    fn from(i: i64) -> Self {
        Number(i as u64)
    }
}

impl From<u64> for Number {
    fn from(u: u64) -> Self {
        Number(u)
    }
}
