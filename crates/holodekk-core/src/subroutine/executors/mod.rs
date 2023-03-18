pub mod local;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum Executor {
    Local(local::LocalExecutor),
}
