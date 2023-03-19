use std::cell::{Ref, RefCell};
use std::collections::HashMap;

use super::{Error, Result, SubroutineDef};
use crate::engine::{Engine, ImageDefKind};

pub struct Service {
    definitions: RefCell<HashMap<String, SubroutineDef>>,
}

impl Service {
    /// Returns a new Subroutine service.
    ///
    /// # Examples
    ///
    /// ```
    /// use holodekkd::subroutines::Service;
    /// let service = Service::new();
    /// ```
    pub fn new() -> Self {
        Self { definitions: RefCell::new(HashMap::new()) }
    }

    /// Initializes the service using the supplied engine.
    pub async fn init(&self, engine: &impl Engine) -> Result<()> {
        for image_def in engine.list_images(ImageDefKind::Subroutine).await {
            self.add_definition(&image_def.name)?;
            if let Some(id) = image_def.id {
                self.add_definition_tag(&image_def.name, &image_def.tag, &id)?;
            }
        }

        Ok(())
    }

    /// Adds a subroutine definition.
    ///
    /// # Examples
    ///
    /// ```
    /// use holodekkd::subroutines::{Service, SubroutineDef};
    /// let service = Service::new();
    /// service.add_definition("acme/widgets");
    /// ```
    pub fn add_definition(&self, name: &str) -> Result<Ref<'_, SubroutineDef>> {
        self.definitions.borrow_mut().insert(
            name.to_string(),
            SubroutineDef::new(name)
        );
        match self.get_definition(name)? {
            Some(sub) => Ok(sub),
            None => Err(Error::Unknown)
        }
    }

    /// Adds a tag to a subroutine definition.
    ///
    /// # Examples
    ///
    /// ```
    /// use holodekkd::subroutines::{Service, SubroutineDef};
    /// let service = Service::new();
    /// service.add_definition("acme/widgets");
    /// service.add_definition_tag("acme/widgets", "latest", "br549");
    ///
    /// ```
     pub fn add_definition_tag(&self, name: &str, tag: &str, engine_id: &str) -> Result<()> {
         match self.definitions.borrow().get(name) {
             Some(sub) => {
                 sub.add_tag(tag, engine_id)
             },
             None => Err(Error::DefinitionNotFound(name.to_string()))
         }
     }

    /// Retrieves a subroutine definition by name.
    ///
    /// # Examples
    ///
    /// ```
    /// use holodekkd::subroutines::{Service, SubroutineDef};
    /// let service = Service::new();
    /// service.add_definition("acme/widgets");
    ///
    /// if let Some(sub) = service.get_definition("acme/widgets").unwrap() {
    ///     println!("tags: {:?}", sub.tags());
    /// };
    /// ```
    pub fn get_definition(&self, name: &str) -> Result<Option<Ref<'_, SubroutineDef>>> {
        let res = Ref::filter_map(
            self.definitions.borrow(),
            |subs| subs.get(name)
        );
        if res.is_err() {
            Ok(None)
        } else {
            Ok(res.ok())
        }
    }

    /// Retrieves the list of available subroutine definitions.
    ///
    /// # Examples
    ///
    /// ```
    /// use holodekkd::subroutines::{Service, SubroutineDef};
    /// let service = Service::new();
    /// service.add_definition("acme/widgets");
    /// service.add_definition_tag("acme/widgets", "latest", "bc1a43fe8d");
    ///
    /// for sub in service.list_definitions().unwrap().iter() {
    ///     println!("Subroutine: {}", sub.name());
    /// }
    /// ```
    pub fn list_definitions(&self) -> Result<Vec<SubroutineDef>> {
        let mut res = vec![];
        // Ok(Ref::map(self.subroutines.borrow(), |subs| subs))
        for (name, sub) in self.definitions.borrow().iter() {
            let ret_sub = SubroutineDef::new(name);
            for tag in sub.tags().iter() {
                let engine_id = sub.get_engine_id(tag)?;
                ret_sub.add_tag(tag, &engine_id)?;
            }
            res.push(ret_sub)
        }
        Ok(res)
    }
}

