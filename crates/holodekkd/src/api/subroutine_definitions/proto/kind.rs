use holodekk::core::subroutine_definitions::entities::SubroutineKind;

use super::enums::RpcSubroutineKind;

impl From<SubroutineKind> for RpcSubroutineKind {
    fn from(kind: SubroutineKind) -> Self {
        match kind {
            SubroutineKind::Ruby => RpcSubroutineKind::Ruby,
            SubroutineKind::Unknown => RpcSubroutineKind::UnknownSubroutineKind,
        }
    }
}

impl From<RpcSubroutineKind> for SubroutineKind {
    fn from(kind: RpcSubroutineKind) -> Self {
        match kind {
            RpcSubroutineKind::Ruby => SubroutineKind::Ruby,
            RpcSubroutineKind::UnknownSubroutineKind => SubroutineKind::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use holodekk::core::subroutine_definitions::entities::SubroutineKind;

    use crate::api::subroutine_definitions::proto::enums::RpcSubroutineKind;

    #[test]
    fn converts_to_ruby_from_rpc_ruby() {
        let rpc_subroutine_kind = RpcSubroutineKind::Ruby;

        let subroutine_kind: SubroutineKind = rpc_subroutine_kind.into();

        assert_eq!(subroutine_kind, SubroutineKind::Ruby);
    }

    #[test]
    fn converts_to_unknown_from_rpc_unknown() {
        let rpc_subroutine_kind = RpcSubroutineKind::UnknownSubroutineKind;

        let subroutine_kind: SubroutineKind = rpc_subroutine_kind.into();

        assert_eq!(subroutine_kind, SubroutineKind::Unknown);
    }

    #[test]
    fn converts_to_rpc_ruby_from_ruby() {
        let subroutine_kind = SubroutineKind::Ruby;

        let rpc_subroutine_kind: RpcSubroutineKind = subroutine_kind.into();

        assert_eq!(rpc_subroutine_kind, RpcSubroutineKind::Ruby);
    }

    #[test]
    fn converts_to_rpc_unknown_from_unknown() {
        let subroutine_kind = SubroutineKind::Unknown;

        let rpc_subroutine_kind: RpcSubroutineKind = subroutine_kind.into();

        assert_eq!(
            rpc_subroutine_kind,
            RpcSubroutineKind::UnknownSubroutineKind
        );
    }
}
