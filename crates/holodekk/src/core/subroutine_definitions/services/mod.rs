mod create;
pub use create::*;

mod get;
pub use get::*;

use std::collections::HashMap;
use std::sync::RwLock;

use crate::core::subroutine_definitions::entities::SubroutineDefinition;

#[derive(Debug)]
pub struct SubroutineDefinitionsService {
    definitions: RwLock<HashMap<String, SubroutineDefinition>>,
}

impl SubroutineDefinitionsService {
    pub fn new(definitions: RwLock<HashMap<String, SubroutineDefinition>>) -> Self {
        Self { definitions }
    }
}
