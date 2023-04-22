mod create;
pub use create::*;

mod find;
pub use find::*;

mod get;
pub use get::*;

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use log::debug;
use walkdir::WalkDir;

use crate::config::HolodekkConfig;
use crate::core::subroutine_definitions::entities::{SubroutineDefinitionEntity, SubroutineKind};

use super::Result;

#[derive(Debug)]
pub struct SubroutineDefinitionsService {
    definitions: RwLock<HashMap<String, SubroutineDefinitionEntity>>,
}

impl SubroutineDefinitionsService {
    pub fn new(definitions: RwLock<HashMap<String, SubroutineDefinitionEntity>>) -> Self {
        Self { definitions }
    }

    pub fn init<C>(config: Arc<C>) -> Result<Self>
    where
        C: HolodekkConfig,
    {
        let definitions = initialize_subroutine_definitions(config)?;

        Ok(Self::new(std::sync::RwLock::new(definitions)))
    }
}

pub fn initialize_subroutine_definitions<C>(
    config: Arc<C>,
) -> Result<HashMap<String, SubroutineDefinitionEntity>>
where
    C: HolodekkConfig,
{
    let mut definitions = HashMap::new();

    for entry in WalkDir::new(config.subroutines_root())
        .min_depth(2)
        .max_depth(2)
    {
        let path = entry.unwrap().path().to_path_buf();
        let name = path
            .strip_prefix(config.subroutines_root())
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let kind = SubroutineKind::detect(&path);

        let definition = SubroutineDefinitionEntity::new(name, path, kind);
        debug!("Loading SubroutineDefinition: {:?}", definition);
        definitions.insert(definition.id().to_owned(), definition);
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
    fn finds_existing_subroutine_definitions() -> std::io::Result<()> {
        let temp = tempdir().unwrap();
        let holodekk_root = temp.into_path();
        let mut data_root = holodekk_root.clone();
        data_root.push("data");
        let mut exec_root = holodekk_root.clone();
        exec_root.push("exec");
        let config = MockConfig::new(&data_root, &exec_root);

        let subroutines_root = config.subroutines_root().to_owned();

        let subroutine_name = "acme/widgets";
        let mut subroutine_path = subroutines_root.clone();
        subroutine_path.push(subroutine_name);
        println!("creating {}", subroutine_path.display());
        std::fs::create_dir_all(&subroutine_path)?;

        let mut manifest_path = subroutine_path.clone();
        manifest_path.push("holodekk.rb");
        std::fs::File::create(&manifest_path)?;

        let definitions = initialize_subroutine_definitions(Arc::new(config)).unwrap();

        assert!(!definitions.is_empty());
        Ok(())
    }
}
