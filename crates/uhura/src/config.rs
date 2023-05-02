use std::path::{Path, PathBuf};

use holodekk::core::ScenePaths;
use holodekk::entities::{SceneEntityId, SceneName};
use holodekk::HolodekkPaths;

#[derive(Clone, Debug)]
pub struct UhuraConfig {
    id: SceneEntityId,
    name: SceneName,
    paths: HolodekkPaths,
    scene_paths: ScenePaths,
}

impl UhuraConfig {
    pub fn new<P>(
        id: &SceneEntityId,
        name: &SceneName,
        data_root: P,
        exec_root: P,
        bin_root: P,
    ) -> Self
    where
        P: AsRef<Path> + Into<PathBuf>,
    {
        let paths = HolodekkPaths::new(data_root.as_ref(), exec_root.as_ref(), bin_root.as_ref());
        let scene_paths = ScenePaths::build(&paths, name);

        Self {
            id: id.to_owned(),
            name: name.to_owned(),
            paths,
            scene_paths,
        }
    }

    pub fn id(&self) -> &SceneEntityId {
        &self.id
    }

    pub fn name(&self) -> &SceneName {
        &self.name
    }

    pub fn paths(&self) -> &HolodekkPaths {
        &self.paths
    }

    pub fn scene_paths(&self) -> &ScenePaths {
        &self.scene_paths
    }
}
