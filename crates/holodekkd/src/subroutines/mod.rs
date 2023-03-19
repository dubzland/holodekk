pub mod api;
mod service;

pub use service::Service;

use std::cell::RefCell;
use std::collections::HashMap;
use std::result;

#[derive(Debug)]
pub enum Error {
    Unknown,
    DefinitionNotFound(String),
    TagNotFound(String),
}

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SubroutineDef {
    name: String,
    tags: RefCell<HashMap<String, String>>,
}

impl SubroutineDef {
    pub fn new(name: &str) -> Self {
        Self { name: name.to_string(), tags: RefCell::new(HashMap::new()) }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn tags(&self) -> Vec<String> {
        self.tags.borrow().keys().map(|x| x.to_owned()).collect()
    }

    pub fn add_tag(&self, tag: &str, engine_id: &str) -> Result<()> {
        self.tags.borrow_mut().insert(tag.to_string(), engine_id.to_string());
        Ok(())
    }

    pub fn get_engine_id(&self, tag: &str) -> Result<String> {
        match self.tags.borrow().get(tag) {
            Some(engine_id) => Ok(engine_id.to_string()),
            None => Err(Error::TagNotFound(tag.to_string())),
        }
    }
}
