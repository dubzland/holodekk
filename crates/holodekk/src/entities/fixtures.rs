use rstest::*;

use crate::entities::SubroutineKind;

use super::*;

#[fixture]
#[once]
pub(crate) fn subroutine() -> Subroutine {
    Subroutine::new(
        "test/sub",
        "/tmp/holodekk/subroutines/test/sub",
        SubroutineKind::Ruby,
    )
}

#[fixture]
#[once]
pub(crate) fn subroutine_with_instance() -> Subroutine {
    let mut sub = Subroutine::new(
        "test/sub",
        "/tmp/holodekk/subroutines/test/sub",
        SubroutineKind::Ruby,
    );
    sub.instances = Some(vec![subroutine_instance(&sub)]);
    sub
}

#[fixture]
#[once]
pub(crate) fn subroutine_instance(subroutine: &Subroutine) -> SubroutineInstance {
    SubroutineInstance::new(
        "test-fleet",
        "test-namespace",
        "/tmp/holodekk/projector/local/subroutines/test/sub",
        &subroutine.id,
    )
}
