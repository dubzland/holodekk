use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Command {
    pub command: String,
    pub description: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LocalExecutor {
    pub build_commands: Vec<Command>,
    pub pre_start_commands: Vec<Command>,
    pub start_command: Command,
    pub working_directory: String,
}
