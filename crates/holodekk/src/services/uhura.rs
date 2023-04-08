use crate::services::Result;

use crate::entities::UhuraStatus;

#[derive(Clone, Copy, Debug, Default)]
pub struct UhuraService {}

impl UhuraService {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn status(&self) -> Result<UhuraStatus> {
        Ok(UhuraStatus {
            pid: std::process::id(),
        })
    }
}
