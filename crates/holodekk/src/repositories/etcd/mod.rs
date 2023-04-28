mod scenes;
pub use scenes::*;

use etcd_client::{Client, Error};

use crate::repositories::EntityId;

pub fn etcd_scene_key(partial: Option<&EntityId>) -> String {
    if let Some(partial) = partial {
        format!("/scenes/{}", partial)
    } else {
        "/scenes/".to_string()
    }
}

pub struct EtcdRepository {
    client: Client,
}

impl EtcdRepository {
    fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn connect(hosts: &[&str]) -> std::result::Result<Self, Error> {
        let client = Client::connect(hosts, None).await?;
        Ok(Self::new(client))
    }
}
