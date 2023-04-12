use std::sync::Arc;

use tokio::sync::mpsc::Sender;

use holodekk::{
    config::HolodekkConfig,
    core::repositories::ProjectorRepository,
    managers::projector::{ProjectorCommand, ProjectorManager},
};

pub struct HolodekkServer {
    projector_manager: ProjectorManager,
}

impl HolodekkServer {
    fn new(projector_manager: ProjectorManager) -> Self {
        Self { projector_manager }
    }

    pub fn start<T>(holodekk_config: Arc<HolodekkConfig>, _repository: Arc<T>) -> Self
    where
        T: ProjectorRepository,
    {
        let projector_manager = ProjectorManager::start(holodekk_config);
        Self::new(projector_manager)
    }

    pub async fn stop(self) -> Result<(), tonic::transport::Error> {
        self.projector_manager.stop().await;
        Ok(())
    }

    pub fn manager_tx(&self) -> Sender<ProjectorCommand> {
        self.projector_manager.cmd_tx()
    }
}
