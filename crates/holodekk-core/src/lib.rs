//! The Holodekk.

mod errors;
pub use errors::{Error, Result};

mod holodekk;
pub use crate::holodekk::*;

pub mod logger;
pub mod utils;
