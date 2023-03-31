mod hello_world {
    tonic::include_proto!("helloworld");
}

pub mod admin {
    mod admin_proto {
        tonic::include_proto!("admin");
    }
    pub mod entities {
        pub use super::admin_proto::{Empty, ProjectorStatus, Subroutine, SubroutineList};
    }

    pub mod core {
        pub use super::admin_proto::core_client::CoreClient;
        pub use super::admin_proto::core_server::{Core, CoreServer};
    }

    pub mod subroutines {
        pub use super::admin_proto::subroutines_client::SubroutinesClient;
        pub use super::admin_proto::subroutines_server::{Subroutines, SubroutinesServer};
    }
}
