use async_trait::async_trait;

use crate::entities::Projector;
use crate::repositories::{ProjectorRepository, Result};

use super::MemoryRepository;

#[async_trait]
impl ProjectorRepository for MemoryRepository {
    async fn projector_create(&self, _projector: Projector) -> Result<Projector> {
        todo!()
    }
}
