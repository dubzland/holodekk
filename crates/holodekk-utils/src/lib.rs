mod api_server;
pub mod libsee;
pub use api_server::{ApiServer, ApiService};
pub mod signals;
pub mod errors {
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
}
