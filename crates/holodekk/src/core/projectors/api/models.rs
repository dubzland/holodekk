use serde::Deserialize;

#[derive(Deserialize)]
pub struct NewProjector {
    pub namespace: String,
}
