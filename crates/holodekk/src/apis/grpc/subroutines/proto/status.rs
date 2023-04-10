use super::entities::{RpcSubroutineStatus, RpcSubroutineStatusCode};
use crate::entities::SubroutineStatus;

impl From<SubroutineStatus> for RpcSubroutineStatus {
    fn from(status: SubroutineStatus) -> Self {
        let mut rpc_status = RpcSubroutineStatus::default();

        match status {
            SubroutineStatus::Unknown => {
                rpc_status.set_code(RpcSubroutineStatusCode::UnknownSubroutineStatus);
            }
            SubroutineStatus::Stopped => {
                rpc_status.set_code(RpcSubroutineStatusCode::Stopped);
            }
            SubroutineStatus::Running(pid) => {
                rpc_status.set_code(RpcSubroutineStatusCode::Running);
                rpc_status.pid = Some(pid as i32);
            }
            SubroutineStatus::Crashed => {
                rpc_status.set_code(RpcSubroutineStatusCode::Crashed);
            }
        }
        rpc_status
    }
}

impl From<RpcSubroutineStatus> for SubroutineStatus {
    fn from(response: RpcSubroutineStatus) -> Self {
        match RpcSubroutineStatusCode::from_i32(response.code) {
            Some(RpcSubroutineStatusCode::Stopped) => SubroutineStatus::Stopped,
            Some(RpcSubroutineStatusCode::Running) => {
                SubroutineStatus::Running(response.pid.unwrap() as u32)
            }
            Some(RpcSubroutineStatusCode::Crashed) => SubroutineStatus::Crashed,
            Some(RpcSubroutineStatusCode::UnknownSubroutineStatus) => SubroutineStatus::Unknown,
            None => SubroutineStatus::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_to_unknown_from_rpc_unknown() {
        let mut rpc_status = RpcSubroutineStatus::default();
        rpc_status.set_code(RpcSubroutineStatusCode::UnknownSubroutineStatus);

        let status: SubroutineStatus = rpc_status.into();
        assert!(matches!(status, SubroutineStatus::Unknown));
    }

    #[test]
    fn converts_to_stopped_from_rpc_stopped() {
        let mut rpc_status = RpcSubroutineStatus::default();
        rpc_status.set_code(RpcSubroutineStatusCode::Stopped);

        let status: SubroutineStatus = rpc_status.into();
        assert!(matches!(status, SubroutineStatus::Stopped));
    }

    #[test]
    fn converts_to_running_from_rpc_running() {
        let mut rpc_status = RpcSubroutineStatus::default();
        rpc_status.set_code(RpcSubroutineStatusCode::Running);
        rpc_status.pid = Some(123);

        let status: SubroutineStatus = rpc_status.into();
        assert!(matches!(status, SubroutineStatus::Running(123)));
    }

    #[test]
    fn converts_to_crashed_from_rpc_crashed() {
        let mut rpc_status = RpcSubroutineStatus::default();
        rpc_status.set_code(RpcSubroutineStatusCode::Crashed);

        let status: SubroutineStatus = rpc_status.into();
        assert!(matches!(status, SubroutineStatus::Crashed));
    }

    #[test]
    fn converts_to_rpc_unknown_from_unknown() {
        let status = SubroutineStatus::Crashed;

        let rpc_status: RpcSubroutineStatus = status.into();
        assert_eq!(rpc_status.code, RpcSubroutineStatusCode::Crashed as i32);
        assert_eq!(rpc_status.pid, None);
    }

    #[test]
    fn converts_to_rpc_stopped_from_stopped() {
        let status = SubroutineStatus::Stopped;

        let rpc_status: RpcSubroutineStatus = status.into();
        assert_eq!(rpc_status.code, RpcSubroutineStatusCode::Stopped as i32);
        assert_eq!(rpc_status.pid, None);
    }

    #[test]
    fn converts_to_rpc_running_from_running() {
        let status = SubroutineStatus::Running(123);

        let rpc_status: RpcSubroutineStatus = status.into();
        assert_eq!(rpc_status.code, RpcSubroutineStatusCode::Running as i32);
        assert_eq!(rpc_status.pid, Some(123));
    }

    #[test]
    fn converts_to_rpc_crashed_from_crashed() {
        let status = SubroutineStatus::Crashed;

        let rpc_status: RpcSubroutineStatus = status.into();
        assert_eq!(rpc_status.code, RpcSubroutineStatusCode::Crashed as i32);
        assert_eq!(rpc_status.pid, None);
    }
}
