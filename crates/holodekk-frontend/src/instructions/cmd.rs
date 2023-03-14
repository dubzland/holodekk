use std::fmt;

pub struct CmdInstruction {
    pub content: String,
}

impl fmt::Display for CmdInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Cmd: {}", self.content)
    }
}

