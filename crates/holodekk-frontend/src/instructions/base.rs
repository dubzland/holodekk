use std::fmt;

pub struct BaseInstruction {
    pub image: String,
}

impl fmt::Display for BaseInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Base image: {}", self.image)
    }
}

