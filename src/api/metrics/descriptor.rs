use crate::api::metrics::{Config, InstrumentKind, NumberKind};

/// Descriptor contains all the settings that describe an instrument, including
/// its name, metric kind, number kind, and the configurable options.
#[derive(Clone, Debug, PartialEq, Hash)]
pub struct Descriptor {
    name: String,
    instrument_kind: InstrumentKind,
    number_kind: NumberKind,
    config: Config,
}

impl Descriptor {
    /// Create a new descriptor
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

    /// The metric instrument's name.
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// The specific kind of instrument.
    pub fn instrument_kind(&self) -> &InstrumentKind {
        &self.instrument_kind
    }

    /// NumberKind returns whether this instrument is declared over int64, float64, or uint64
    /// values.
    pub fn number_kind(&self) -> &NumberKind {
        &self.number_kind
    }

    /// A human-readable description of the metric instrument.
    pub fn description(&self) -> Option<&String> {
        self.config.description.as_ref()
    }

    /// Assign a new description
    pub fn set_description(&mut self, description: String) {
        self.config.description = Some(description);
    }

    /// Unit describes the units of the metric instrument.
    pub fn unit(&self) -> Option<&str> {
        self.config.unit.as_ref().map(|unit| unit.as_ref())
    }

    /// LibraryName returns the metric instrument's library name, typically
    /// given via a call to Provider.Meter().
    pub fn library_name(&self) -> &str {
        self.config.library_name.as_str()
    }
}
