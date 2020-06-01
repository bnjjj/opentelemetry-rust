use crate::api::metrics::{Config, InstrumentKind, NumberKind};

/// TODO
#[derive(Debug)]
pub struct Descriptor {
    pub(crate) name: String,
    pub(crate) instrument_kind: InstrumentKind,
    pub(crate) number_kind: NumberKind,
    pub(crate) config: Config,
}

impl Descriptor {
    /// TODO
    pub fn new(
        name: String,
        library_name: String,
        instrument_kind: InstrumentKind,
        number_kind: NumberKind,
    ) -> Self {
        Descriptor {
            name,
            instrument_kind,
            number_kind,
            config: Config::with_library_name(library_name),
        }
    }
}
