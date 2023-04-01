//! The Holodekk.

pub mod api;
mod errors;
pub use errors::{Error, Result};

mod holodekk;
pub use crate::holodekk::*;

pub mod logger;
