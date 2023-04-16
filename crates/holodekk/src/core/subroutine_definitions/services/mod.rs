mod create;
pub use create::*;

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use log::debug;
use walkdir::WalkDir;

use crate::config::HolodekkConfig;
use crate::core::services::Result;
use crate::core::subroutine_definitions::entities::{SubroutineDefinition, SubroutineKind};

#[derive(Debug)]
pub struct SubroutineDefinitionsService {
    definitions: RwLock<HashMap<String, SubroutineDefinition>>,
}

impl SubroutineDefinitionsService {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn init<C>(config: Arc<C>) -> Result<SubroutineDefinitionsService>
    where
        C: HolodekkConfig,
    {
        let definitions = init_subroutine_definitions(config)?;
        Ok(Self {
            definitions: RwLock::new(definitions),
        })
    }
}

impl Default for SubroutineDefinitionsService {
    fn default() -> Self {
        Self {
            definitions: RwLock::new(HashMap::new()),
        }
    }
}

fn init_subroutine_definitions<C>(config: Arc<C>) -> Result<HashMap<String, SubroutineDefinition>>
where
    C: HolodekkConfig,
{
    let mut definitions = HashMap::new();

    for entry in WalkDir::new(config.paths().subroutines())
        .min_depth(2)
        .max_depth(2)
    {
        let path = entry.unwrap().path().to_path_buf();
        let name = path
            .strip_prefix(config.paths().subroutines())
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let kind = SubroutineKind::detect(&path);

        let definition = SubroutineDefinition::new(name, path, kind);
        debug!("Loading SubroutineDefinition: {:?}", definition);
        definitions.insert(definition.name().to_owned(), definition);
    }

    Ok(definitions)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use tempfile::tempdir;

    use crate::config::fixtures::MockConfig;

    use super::*;

    #[test]
    fn finds_existing_subroutines() -> std::io::Result<()> {
        let temp = tempdir().unwrap();
        let holodekk_root = temp.into_path();
        let config = MockConfig::new(&holodekk_root);

        let subroutines_root = config.paths().subroutines().to_owned();

        let subroutine_name = "acme/widgets";
        let mut subroutine_path = subroutines_root.clone();
        subroutine_path.push(subroutine_name);
        println!("creating {}", subroutine_path.display());
        std::fs::create_dir_all(&subroutine_path)?;

        let mut manifest_path = subroutine_path.clone();
        manifest_path.push("holodekk.rb");
        std::fs::File::create(&manifest_path)?;

        let definitions = init_subroutine_definitions(Arc::new(config)).unwrap();

        assert!(!definitions.is_empty());
        Ok(())
    }
}
