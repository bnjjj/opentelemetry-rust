use crate::api::Unit;

/// Config contains some options for metrics of any kind.
#[derive(Clone, Debug, PartialEq, Hash)]
pub struct Config {
    pub(crate) description: Option<String>,
    pub(crate) unit: Option<Unit>,
    pub(crate) library_name: String,
}

impl Config {
    /// Create a new config from library name
    pub fn with_library_name(library_name: String) -> Self {
        Config {
            description: None,
            unit: None,
            library_name,
        }
    }

    /// Description is an optional field describing the metric instrument.
    pub fn description(&self) -> Option<&String> {
        self.description.as_ref()
    }

    /// Unit is an optional field describing the metric instrument data.
    pub fn unit(&self) -> Option<&Unit> {
        self.unit.as_ref()
    }

    /// Library name is the name given to the Meter that created this instrument.
    pub fn library_name(&self) -> &String {
        &self.library_name
    }
}
