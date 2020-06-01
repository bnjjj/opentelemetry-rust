use crate::api::Unit;

/// TODO
#[derive(Debug)]
pub struct Config {
    pub(crate) description: Option<String>,
    pub(crate) unit: Option<Unit>,
    pub(crate) library_name: String,
}

impl Config {
    /// TODO
    pub fn with_library_name(library_name: String) -> Self {
        Config {
            description: None,
            unit: None,
            library_name,
        }
    }

    /// TODO
    pub fn description(&self) -> Option<&String> {
        self.description.as_ref()
    }

    /// TODO
    pub fn unit(&self) -> Option<&Unit> {
        self.unit.as_ref()
    }

    /// TODO
    pub fn library_name(&self) -> &String {
        &self.library_name
    }
}
