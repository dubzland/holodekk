use std::fmt;

pub struct UnknownInstruction {
    pub content: String,
}

impl fmt::Display for UnknownInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Unknown: {}", self.content)
    }
}

pub struct UnknownMiscInstruction {
    pub tag: String,
    pub content: String,
}

impl fmt::Display for UnknownMiscInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Unknown Misc: {}: {}", self.tag, self.content)
    }
}
