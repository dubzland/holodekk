use std::fmt;

pub struct AddInstruction {
    pub sources: Vec<String>,
    pub target: String,
}

impl fmt::Display for AddInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Copy [\n")?;
        for source in &self.sources {
            write!(f, "  {} TO {}\n", source, self.target)?;
        }
        write!(f, "]")
    }
}

