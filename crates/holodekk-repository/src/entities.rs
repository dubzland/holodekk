use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Subroutine {
    id: Option<i32>,
    name: String,
    path: PathBuf,
    pid: i32,
}
