use crate::api::metrics::{Config, InstrumentKind, NumberKind};

/// TODO
#[derive(Clone, Debug, PartialEq, Hash)]
pub struct Descriptor {
    name: String,
    instrument_kind: InstrumentKind,
    number_kind: NumberKind,
    config: Config,
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

    /// TODO
    pub fn name(&self) -> &String {
        &self.name
    }

    /// TODO
    pub fn instrument_kind(&self) -> &InstrumentKind {
        &self.instrument_kind
    }

    /// TODO
    pub fn number_kind(&self) -> &NumberKind {
        &self.number_kind
    }

    /// TODO
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// TODO
    pub fn config_mut(&mut self) -> &mut Config {
        &mut self.config
    }
}
