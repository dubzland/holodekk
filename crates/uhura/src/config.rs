use std::path::Path;

use holodekk::scene;
use holodekk::Paths;

#[derive(Clone, Debug)]
pub struct Config {
    id: scene::entity::Id,
    name: scene::entity::Name,
    paths: Paths,
    scene_paths: scene::Paths,
}

impl Config {
    pub fn new<P>(
        id: &scene::entity::Id,
        name: &scene::entity::Name,
        data_root: &P,
        exec_root: &P,
        bin_root: &P,
    ) -> Self
    where
        P: AsRef<Path>,
    {
        let paths = Paths::new(data_root.as_ref(), exec_root.as_ref(), bin_root.as_ref());
        let scene_paths = scene::Paths::build(&paths, name);

        Self {
            id: id.clone(),
            name: name.clone(),
            paths,
            scene_paths,
        }
    }

    #[must_use]
    pub fn id(&self) -> &scene::entity::Id {
        &self.id
    }

    #[must_use]
    pub fn name(&self) -> &scene::entity::Name {
        &self.name
    }

    #[must_use]
    pub fn paths(&self) -> &Paths {
        &self.paths
    }

    #[must_use]
    pub fn scene_paths(&self) -> &scene::Paths {
        &self.scene_paths
    }
}
