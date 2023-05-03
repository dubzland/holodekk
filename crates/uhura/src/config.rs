use std::path::Path;

use holodekk::entities::{SceneEntityId, SceneName};
use holodekk::{HolodekkPaths, ScenePaths};

#[derive(Clone, Debug)]
pub struct Config {
    id: SceneEntityId,
    name: SceneName,
    paths: HolodekkPaths,
    scene_paths: ScenePaths,
}

impl Config {
    pub fn new<P>(
        id: &SceneEntityId,
        name: &SceneName,
        data_root: &P,
        exec_root: &P,
        bin_root: &P,
    ) -> Self
    where
        P: AsRef<Path>,
    {
        let paths = HolodekkPaths::new(data_root.as_ref(), exec_root.as_ref(), bin_root.as_ref());
        let scene_paths = ScenePaths::build(&paths, name);

        Self {
            id: id.clone(),
            name: name.clone(),
            paths,
            scene_paths,
        }
    }

    #[must_use]
    pub fn id(&self) -> &SceneEntityId {
        &self.id
    }

    #[must_use]
    pub fn name(&self) -> &SceneName {
        &self.name
    }

    #[must_use]
    pub fn paths(&self) -> &HolodekkPaths {
        &self.paths
    }

    #[must_use]
    pub fn scene_paths(&self) -> &ScenePaths {
        &self.scene_paths
    }
}
