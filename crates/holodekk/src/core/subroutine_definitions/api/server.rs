use std::sync::Arc;

use crate::core::subroutine_definitions::services::CreateSubroutineDefinition;

pub trait SubroutineDefinitionsApiServices<D> {
    fn definitions(&self) -> Arc<D>
    where
        D: CreateSubroutineDefinition + Send + Sync + 'static;
}
