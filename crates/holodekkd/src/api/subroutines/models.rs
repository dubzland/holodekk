use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct NewSubroutine {
    pub subroutine_definition_id: String,
}
