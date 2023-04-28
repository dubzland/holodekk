//! The Holodekk core
//!
//! Holodekk is built loosely following the principles describe in the [Onion
//! Architecture](https://jeffreypalermo.com/2008/07/the-onion-architecture-part-1/).  This module
//! represents the very center of that onion.
//!
//! # Layout
//! Each type of resource being managed by the Holodekk is represented by its own module, which is
//! then roughly divided into the following components:
//!
//! ## Entity
//! Struct object containing the actual resource attributes.  Each module will contain at least one
//! entity, though some contain more than one (when relationships are tightly coupled).  Entities
//! are persited to a backing store by way of repositories, described below.
//!
//! ## Service
//! These are the public-facing API meant be consumed by anything outside of the core.  This is
//! where business rules are applied and data management occurs.  Analgous to controllers in the
//! MVC architecture, they are intended to be the glue between the outer application and the
//! entities residing in repositories.
//!
//! ## Worker
//! Most entities definied within the system represent external items (processes, networks, etc)
//! that must be managed (in addition to the actual data store).  Workers accomplish this by
//! accepting requests from services and performing the necessary background tasks (creating a
//! network, launching a container, etc).
//!
//! ## Repository
//! This is an abstract representation of the data access requirements of the resources the module
//! is responsible for (Traits in rust parlance).  Each repository is intended to be implemented by
//! some sort of concrete backing store (etcd, Postgres, etc), allowing the actual storage system
//! to be transparent to the rest of the core.
//!
// pub mod containers;
pub mod entities;
pub mod enums;
pub mod repositories;
pub mod scene_create;
pub mod scene_delete;
pub mod scene_get;
pub mod scenes_find;
pub mod subroutine_create;
pub mod subroutine_delete;
pub mod subroutine_get;
pub mod subroutines_find;
// pub mod workers;
// pub mod scenes;
// pub mod subroutine_definitions;
// pub mod subroutines;

use std::path::PathBuf;
use std::sync::Arc;

use holodekk_common::config::HolodekkPaths;

use entities::{SceneEntity, SubroutineEntity};

#[derive(Debug)]
pub struct ScenePaths {
    root: PathBuf,
    pidfile: PathBuf,
    socket: PathBuf,
}

impl ScenePaths {
    pub fn build(paths: Arc<HolodekkPaths>, scene: &SceneEntity) -> Self {
        let mut root = paths.projectors_root().clone();
        root.push(&scene.id);

        let mut pidfile = root.clone();
        pidfile.push("uhura.pid");

        let mut socket = root.clone();
        socket.push("uhura.sock");

        Self {
            root,
            pidfile,
            socket,
        }
    }

    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    pub fn pidfile(&self) -> &PathBuf {
        &self.pidfile
    }

    pub fn socket(&self) -> &PathBuf {
        &self.socket
    }
}

#[derive(Debug)]
pub struct SubroutinePaths {
    root: PathBuf,
    pidfile: PathBuf,
    logfile: PathBuf,
    socket: PathBuf,
}

impl SubroutinePaths {
    pub fn build(paths: Arc<HolodekkPaths>, subroutine: &SubroutineEntity) -> Self {
        let mut root = paths.subroutines_root().clone();
        root.push(subroutine.id.clone());

        let mut pidfile = root.clone();
        pidfile.push("subroutine.pid");

        let mut logfile = root.clone();
        logfile.push("subroutine.log");

        let mut socket = root.clone();
        socket.push("log.sock");

        Self {
            root,
            pidfile,
            logfile,
            socket,
        }
    }

    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    pub fn pidfile(&self) -> &PathBuf {
        &self.pidfile
    }

    pub fn logfile(&self) -> &PathBuf {
        &self.logfile
    }

    pub fn socket(&self) -> &PathBuf {
        &self.socket
    }
}
