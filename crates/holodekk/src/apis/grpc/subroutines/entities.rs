use std::path::PathBuf;

use crate::entities::{Subroutine, SubroutineStatus};

pub use super::proto::entities::{RpcEmpty, RpcSubroutine, RpcSubroutineList, RpcSubroutineStatus};

impl From<RpcSubroutine> for Subroutine {
    fn from(subroutine: RpcSubroutine) -> Self {
        let status = match RpcSubroutineStatus::from_i32(subroutine.status) {
            Some(RpcSubroutineStatus::Stopped) => SubroutineStatus::Stopped,
            Some(RpcSubroutineStatus::Running) => SubroutineStatus::Running(subroutine.pid as u32),
            Some(RpcSubroutineStatus::Crashed) => SubroutineStatus::Crashed,
            None => SubroutineStatus::Stopped,
        };
        Self {
            name: subroutine.name,
            path: PathBuf::from(subroutine.path),
            status,
        }
    }
}
