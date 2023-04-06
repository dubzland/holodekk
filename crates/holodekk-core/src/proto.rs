pub mod common {
    tonic::include_proto!("common");
}

pub(crate) mod applications {
    tonic::include_proto!("applications");
}

pub(crate) mod subroutines {
    tonic::include_proto!("subroutines");
}
