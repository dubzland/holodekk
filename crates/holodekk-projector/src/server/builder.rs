use super::{ProjectorServer, Service};

#[derive(Default)]
pub struct ProjectorServerBuilder {
    namespace: Option<String>,
    admin_service: Option<Service>,
    projector_service: Option<Service>,
}

impl ProjectorServerBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn for_namespace(&mut self, namespace: &str) -> &mut Self {
        self.namespace.replace(namespace.to_string());
        self
    }

    pub fn with_admin_service(&mut self, service: Service) -> &mut Self {
        self.admin_service.replace(service);
        self
    }

    pub fn with_projector_service(&mut self, service: Service) -> &mut Self {
        self.projector_service.replace(service);
        self
    }

    pub fn build(&mut self) -> ProjectorServer {
        let namespace = self.namespace.take().unwrap();
        let admin_service = self.admin_service.take().unwrap();
        let projector_service = self.projector_service.take().unwrap();
        ProjectorServer::new(&namespace, admin_service, projector_service)
    }
}
