use crate::entities::UhuraStatus;

#[derive(Clone, Copy, Debug, Default)]
pub struct Service {}

impl Service {
    #[must_use]
    pub fn new() -> Self {
        Service::default()
    }

    #[must_use]
    pub fn status(&self) -> UhuraStatus {
        UhuraStatus {
            pid: std::process::id(),
        }
    }
}
