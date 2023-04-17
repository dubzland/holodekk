use serde::{Deserialize, Serialize};

use crate::core::subroutine_definitions::entities::SubroutineKind;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct NewSubroutineDefinition {
    pub name: String,
    pub path: String,
    pub kind: SubroutineKind,
}
