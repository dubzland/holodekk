pub mod api;
pub mod config;
pub mod holodekk;
// pub mod errors;
pub mod scene;

// pub fn initialize_subroutine_definitions(
//     paths: Arc<HolodekkPaths>,
// ) -> HashMap<String, SubroutineDefinitionEntity> {
//     let mut definitions = HashMap::new();

//     let mut subroutines_root = paths.data_root().to_owned();
//     subroutines_root.push("subroutines");

//     for entry in WalkDir::new(&subroutines_root).min_depth(2).max_depth(2) {
//         let path = entry.unwrap().path().to_path_buf();
//         let name = path
//             .strip_prefix(&subroutines_root)
//             .unwrap()
//             .to_str()
//             .unwrap()
//             .to_string();
//         let kind = SubroutineKind::detect(&path);

//         let definition = SubroutineDefinitionEntity::new(name, path, kind);
//         debug!("Loading SubroutineDefinition: {:?}", definition);
//         definitions.insert(definition.id().to_owned(), definition);
//     }

//     definitions
// }

// #[cfg(test)]
// mod tests {
//     use std::path::PathBuf;
//     use std::sync::Arc;

//     use tempfile::tempdir;

//     use super::*;

//     #[test]
//     fn finds_existing_subroutine_definitions() -> std::io::Result<()> {
//         let temp = tempdir().unwrap();
//         let holodekk_root = temp.into_path();
//         let mut data_root = holodekk_root.clone();
//         data_root.push("data");
//         let mut exec_root = holodekk_root.clone();
//         exec_root.push("exec");
//         let paths = HolodekkPaths::new(&data_root, &exec_root, &PathBuf::from("/usr/local/bin"));

//         let mut subroutine_definitions_root = paths.data_root().to_owned();
//         subroutine_definitions_root.push("subroutines");

//         let subroutine_name = "acme/widgets";
//         let mut subroutine_path = subroutine_definitions_root.clone();
//         subroutine_path.push(subroutine_name);
//         println!("creating {}", subroutine_path.display());
//         std::fs::create_dir_all(&subroutine_path)?;

//         let mut manifest_path = subroutine_path.clone();
//         manifest_path.push("holodekk.rb");
//         std::fs::File::create(&manifest_path)?;

//         let definitions = initialize_subroutine_definitions(Arc::new(paths));

//         assert!(!definitions.is_empty());
//         Ok(())
//     }
// }
