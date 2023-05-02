pub mod etcd;
pub mod memory;

use clap::ValueEnum;

#[derive(Clone, Copy, Debug, PartialEq, ValueEnum)]
pub enum RepositoryKind {
    Etcd,
    Memory,
}
