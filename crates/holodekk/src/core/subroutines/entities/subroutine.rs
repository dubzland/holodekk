use serde::{Deserialize, Serialize};

use crate::core::projectors::entities::ProjectorEntity;
use crate::core::subroutine_definitions::entities::SubroutineDefinitionEntity;
use crate::utils::generate_id;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub enum SubroutineStatus {
    Unknown,
    Stopped,
    Running(u32),
    Crashed,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct SubroutineEntity {
    id: String,
    status: SubroutineStatus,
    projector_id: String,
    subroutine_definition_id: String,
}

impl SubroutineEntity {
    pub fn new(
        id: String,
        status: SubroutineStatus,
        projector_id: String,
        subroutine_definition_id: String,
    ) -> Self {
        Self {
            id,
            status,
            projector_id,
            subroutine_definition_id,
        }
    }

    pub fn build(
        projector: &ProjectorEntity,
        subroutine_definition: &SubroutineDefinitionEntity,
    ) -> Self {
        let id = generate_id();

        Self::new(
            id,
            SubroutineStatus::Unknown,
            projector.id().into(),
            subroutine_definition.id().into(),
        )
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn status(&self) -> SubroutineStatus {
        self.status
    }

    pub fn projector_id(&self) -> &str {
        &self.projector_id
    }

    pub fn subroutine_definition_id(&self) -> &str {
        &self.subroutine_definition_id
    }

    pub fn set_status(&mut self, status: SubroutineStatus) {
        self.status = status;
    }
}
