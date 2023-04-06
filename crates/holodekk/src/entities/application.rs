use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Application {
    name: String,
}

impl Default for Application {
    fn default() -> Self {
        Self {
            name: "".to_string(),
        }
    }
}

impl Application {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
