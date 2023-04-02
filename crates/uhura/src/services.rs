use crate::entities::ProjectorStatus;

#[derive(Clone, Copy, Debug, Default)]
pub struct CoreService {}

impl CoreService {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn status(&self) -> std::result::Result<ProjectorStatus, std::io::Error> {
        Ok(ProjectorStatus {
            pid: std::process::id(),
        })
    }
}
