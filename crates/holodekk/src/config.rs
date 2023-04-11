use std::path::PathBuf;

#[derive(Clone, Debug, PartialEq)]
pub struct HolodekkConfig {
    pub fleet: String,
    pub root_path: PathBuf,
    pub bin_path: PathBuf,
}

#[cfg(test)]
pub mod fixtures {
    use rstest::*;

    use super::*;

    #[fixture]
    pub fn holodekk_config() -> HolodekkConfig {
        HolodekkConfig {
            fleet: "test".to_string(),
            root_path: "/tmp".into(),
            bin_path: "/bmp".into(),
        }
    }
}
