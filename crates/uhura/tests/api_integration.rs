use holodekk_utils::ApiServer;
use tempfile::tempdir;

use uhura::{
    api::{client::UhuraClient, server::UhuraApi},
    services::CoreService,
};

#[cfg(test)]
mod test {
    use super::*;
    use std::path::{Path, PathBuf};
    use tokio::task::JoinHandle;

    fn setup_test_logger() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    async fn setup_uhura_server<P: AsRef<Path>>(
        root: P,
    ) -> (
        ApiServer<UhuraApi>,
        JoinHandle<std::result::Result<(), tonic::transport::Error>>,
        PathBuf,
    ) {
        let socket = root.as_ref().to_owned().join("uhura.socket");
        let core_service = CoreService::new();
        let api_service = UhuraApi::new(core_service);
        let api_server = ApiServer::listen_uds(api_service, &socket);
        let handle = api_server.start();
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        (api_server, handle, socket)
    }

    async fn setup_uhura_client<P: AsRef<Path>>(
        socket: P,
    ) -> std::result::Result<UhuraClient, uhura::api::client::Error> {
        UhuraClient::connect_uds(socket.as_ref()).await
    }

    #[tokio::test]
    async fn status_returns_valid_pid() {
        setup_test_logger();
        let root = tempdir().unwrap();

        let (server, handle, socket) = setup_uhura_server(root.path()).await;
        let client = setup_uhura_client(&socket).await.unwrap();
        let result = client.core().status().await.unwrap();
        server.stop();
        assert_eq!(result.pid, std::process::id());
        handle.await.unwrap().unwrap();
    }
}
