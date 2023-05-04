//! Error utilities

pub mod grpc;

use std::{error, fmt};

/// Provides a consistent formatting for all Debug-based error displays
///
/// # Errors
///
/// see [`std::fmt::Display`]
pub fn error_chain_fmt(e: &impl error::Error, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "{e}\n")?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by:\n\t{cause}")?;
        current = cause.source();
    }
    Ok(())
}
