use holodekk_projector::api::server::ApplicationsService;
use holodekk_utils::ApiServer;

use super::ProjectorServer;

use crate::api::server::UhuraApi;

#[derive(Debug)]
pub struct ProjectorServerBuilder {
    namespace: Option<String>,
    uhura_api: Option<ApiServer<UhuraApi>>,
    projector_api: Option<ApiServer<ApplicationsService>>,
}

impl Default for ProjectorServerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ProjectorServerBuilder {
    pub fn new() -> Self {
        Self {
            namespace: None,
            uhura_api: None,
            projector_api: None,
        }
    }

    pub fn for_namespace(&mut self, namespace: &str) -> &mut Self {
        self.namespace.replace(namespace.to_string());
        self
    }

    pub fn with_uhura_api(&mut self, api: ApiServer<UhuraApi>) -> &mut Self {
        self.uhura_api.replace(api);
        self
    }

    pub fn with_projector_api(&mut self, api: ApiServer<ApplicationsService>) -> &mut Self {
        self.projector_api.replace(api);
        self
    }

    pub fn build(&mut self) -> ProjectorServer {
        let namespace = self.namespace.take().unwrap();
        let uhura_api = self.uhura_api.take().unwrap();
        let projector_api = self.projector_api.take().unwrap();
        ProjectorServer::new(&namespace, uhura_api, projector_api)
    }
}
