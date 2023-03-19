use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::io;
use std::result;

use super::Subroutine;

#[derive(Debug, PartialEq)]
pub enum Error {
    NotFound,
    NameInvalid,
    IoError,
    WalkError,
}

#[derive(Debug)]
pub enum StoreKind {
    FileSystem
}

impl From<io::Error> for Error {
    fn from(_err: io::Error) -> Error {
        Error::IoError
    }
}

impl From<walkdir::Error> for Error {
    fn from(_err: walkdir::Error) -> Error {
        Error::WalkError
    }
}

pub type Result<T> = result::Result<T, Error>;

pub struct Store {
    subroutines: RefCell<HashMap<String, Subroutine>>,
}

impl Store {
    /// Returns a new Subroutine store.
    ///
    /// # Examples
    ///
    /// ```
    /// use holodekkd::subroutines::Store;
    /// let store = Store::new();
    /// ```
    pub fn new() -> Self {
        Self { subroutines: RefCell::new(HashMap::new()) }
    }

    /// Adds a subroutine to the store.
    ///
    /// # Examples
    ///
    /// ```
    /// use holodekkd::subroutines::{Store, Subroutine};
    /// let store = Store::new();
    /// store.add("acme/widgets");
    /// ```
    pub fn add(&self, name: &str) -> Result<Ref<'_, Subroutine>> {
        self.subroutines.borrow_mut().insert(name.to_string(), Subroutine::new(name));
        match self.get(name)? {
            Some(sub) => Ok(sub),
            None => Err(Error::NotFound)
        }
        // Ok(())
    }

    /// Adds a tag to a subroutine.
    ///
    /// # Examples
    ///
    /// ```
    /// use holodekkd::subroutines::{Store, Subroutine};
    /// let store = Store::new();
    /// store.add("acme/widgets");
    ///
    /// ```
     pub fn add_tag(&self, name: &str, tag: &str, engine_id: &str) -> Result<()> {
         match self.subroutines.borrow().get(name) {
             Some(sub) => {
                 sub.add_tag(tag, engine_id)
             },
             None => Err(Error::NotFound)
         }
     }


    /// Retrieves a subroutine from the store.
    ///
    /// # Examples
    ///
    /// ```
    /// use holodekkd::subroutines::{Store, Subroutine};
    /// let store = Store::new();
    /// store.add("acme/widgets");
    ///
    /// if let Some(sub) = store.get("acme/widgets").unwrap() {
    ///     println!("tags: {:?}", sub.tags());
    /// };
    /// ```
    pub fn get(&self, name: &str) -> Result<Option<Ref<'_, Subroutine>>> {
        let res = Ref::filter_map(
            self.subroutines.borrow(),
            |subs| subs.get(name)
        );
        if res.is_err() {
            Ok(None)
        } else {
            Ok(res.ok())
        }
    }

    /// Retrieves a subroutine from the store.
    ///
    /// # Examples
    ///
    /// ```
    /// use holodekkd::subroutines::{Store, Subroutine};
    /// let store = Store::new();
    /// store.add("acme/widgets");
    /// store.add_tag("acme/widgets", "latest", "bc1a43fe8d");
    ///
    /// for sub in store.list().unwrap().iter() {
    ///     println!("Subroutine: {}", sub.name());
    /// }
    /// ```
    pub fn list(&self) -> Result<Vec<Subroutine>> {
        let mut res = vec![];
        // Ok(Ref::map(self.subroutines.borrow(), |subs| subs))
        for (name, sub) in self.subroutines.borrow().iter() {
            let ret_sub = Subroutine::new(name);
            for tag in sub.tags().iter() {
                let engine_id = sub.get_engine_id(tag)?;
                ret_sub.add_tag(tag, &engine_id)?;
            }
            res.push(ret_sub)
        }
        Ok(res)
    }
}
