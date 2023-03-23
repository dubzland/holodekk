//! The Projector implementation for the Holodekk.
//!
//! The projector is the glue between the Holodekk platform and the subroutines it runs.
//! Subroutines make requests of the Projector, and the Projector keeps them up to date with the
//! current state of the system.

pub mod client;
pub mod error;
pub mod server;
