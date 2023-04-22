use crate::entities::UhuraStatus;

#[derive(Clone, Copy, Debug, Default)]
pub struct UhuraService {}

impl UhuraService {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn status(&self) -> UhuraStatus {
        UhuraStatus {
            pid: std::process::id(),
        }
    }
}
