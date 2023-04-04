use holodekk_utils::server::{
    tonic::{TonicServer, TonicServerHandle},
    Server, ServerHandle,
};
use tempfile::tempdir;

use uhura::{
    api::{client::UhuraClient, server::UhuraApi},
    services::CoreService,
};

#[cfg(test)]
mod test {
    use super::*;
    use std::path::{Path, PathBuf};

    fn setup_test_logger() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    async fn launch_uhura_server<P: AsRef<Path>>(
        root: P,
    ) -> (
        TonicServer,
        TonicServerHandle,
        // JoinHandle<std::result::Result<(), tonic::transport::Error>>,
        PathBuf,
    ) {
        let socket = root.as_ref().to_owned().join("uhura.socket");
        let core_service = CoreService::new();
        let api = UhuraApi::new(core_service).build().listen_uds(&socket);
        let mut handle = api.listen();
        handle.start();

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        (api, handle, socket)
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

        let (_server, mut handle, socket) = launch_uhura_server(root.path()).await;
        let client = setup_uhura_client(&socket).await.unwrap();
        let result = client.core().status().await.unwrap();
        handle.stop().await.unwrap();
        assert_eq!(result.pid, std::process::id());
    }
}
