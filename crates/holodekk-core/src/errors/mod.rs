pub mod grpc;
mod holodekk;
pub use self::holodekk::HolodekkError;

use std::{error, fmt};

pub fn error_chain_fmt(e: &impl error::Error, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "{}\n", e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by:\n\t{}", cause)?;
        current = cause.source();
    }
    Ok(())
}
