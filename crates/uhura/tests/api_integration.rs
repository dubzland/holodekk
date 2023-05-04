use std::path::{Path, PathBuf};
use std::sync::Arc;

use tempfile::tempdir;

use holodekk::errors::grpc::ClientResult;
use holodekk::utils::{
    server::{grpc, Grpc, Handle},
    ConnectionInfo, Server,
};

use uhura::apis::grpc::uhura::uhura_api;

#[cfg(test)]
mod test {
    use super::*;

    fn setup_test_logger() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    async fn launch_uhura_server<P: AsRef<Path>>(root: P) -> (grpc::Handle, PathBuf) {
        let socket = root.as_ref().to_owned().join("uhura.socket");
        let config = ConnectionInfo::unix(&socket);

        let uhura_service = Arc::new(uhura::Service::new());
        let uhura_server =
            tonic::transport::Server::builder().add_service(uhura_api(uhura_service));
        let handle = Grpc::start(&config, uhura_server);

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        (handle, socket)
    }

    async fn setup_uhura_client<P: AsRef<Path>>(socket: P) -> ClientResult<uhura::Client> {
        uhura::Client::connect_unix(socket.as_ref()).await
    }

    #[tokio::test]
    async fn status_returns_valid_pid() {
        setup_test_logger();
        let root = tempdir().unwrap();

        let (handle, socket) = launch_uhura_server(root.path()).await;
        let client = setup_uhura_client(&socket).await.unwrap();
        let result = client.uhura().status().await.unwrap();
        handle.stop().await.unwrap();
        assert_eq!(result.pid, std::process::id());
    }
}
