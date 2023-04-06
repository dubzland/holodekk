use holodekk::services::Result;

use crate::entities::ProjectorStatus;

#[derive(Clone, Copy, Debug, Default)]
pub struct CoreService {}

impl CoreService {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn status(&self) -> Result<ProjectorStatus> {
        Ok(ProjectorStatus {
            pid: std::process::id(),
        })
    }
}
