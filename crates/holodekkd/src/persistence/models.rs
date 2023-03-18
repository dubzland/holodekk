pub struct Subroutine {
    name: String,
}

impl Subroutine {
    pub fn new(name: &str) -> Self {
        Self { name: name.to_string() }
    }
}

