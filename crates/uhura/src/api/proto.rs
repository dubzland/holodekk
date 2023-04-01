mod uhura_proto {
    tonic::include_proto!("uhura");
}

pub mod entities {
    pub use super::uhura_proto::{Empty, ProjectorStatus, Subroutine, SubroutineList};
}

pub mod core {
    pub use super::uhura_proto::core_client::CoreClient;
    pub use super::uhura_proto::core_server::{Core, CoreServer};
}

pub mod subroutines {
    pub use super::uhura_proto::subroutines_client::SubroutinesClient;
    pub use super::uhura_proto::subroutines_server::{Subroutines, SubroutinesServer};
}
