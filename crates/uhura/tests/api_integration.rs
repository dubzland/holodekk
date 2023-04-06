use std::path::{Path, PathBuf};
use std::sync::Arc;

use futures_util::FutureExt;
use tempfile::tempdir;
use tokio::{
    net::UnixListener,
    sync::oneshot::{channel, Sender},
    task::JoinHandle,
};
use tokio_stream::wrappers::UnixListenerStream;

use holodekk::errors::grpc::GrpcClientResult;

use uhura::{
    api::{client::UhuraClient, server::core_api},
    services::CoreService,
};

#[cfg(test)]
mod test {
    use super::*;

    fn setup_test_logger() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    async fn launch_uhura_server<P: AsRef<Path>>(
        root: P,
    ) -> (
        Sender<()>,
        JoinHandle<std::result::Result<(), tonic::transport::Error>>,
        PathBuf,
    ) {
        let (shutdown_tx, shutdown_rx) = channel();
        let socket = root.as_ref().to_owned().join("uhura.socket");
        let uds = UnixListener::bind(&socket).unwrap();
        let listener = UnixListenerStream::new(uds);

        let core_service = Arc::new(CoreService::new());
        let handle = tokio::spawn(async move {
            tonic::transport::Server::builder()
                .add_service(core_api(core_service))
                .serve_with_incoming_shutdown(listener, shutdown_rx.map(drop))
                .await
        });

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        (shutdown_tx, handle, socket)
    }

    async fn setup_uhura_client<P: AsRef<Path>>(socket: P) -> GrpcClientResult<UhuraClient> {
        UhuraClient::connect_uds(socket.as_ref()).await
    }

    #[tokio::test]
    async fn status_returns_valid_pid() {
        setup_test_logger();
        let root = tempdir().unwrap();

        let (shutdown_tx, handle, socket) = launch_uhura_server(root.path()).await;
        let client = setup_uhura_client(&socket).await.unwrap();
        let result = client.core().status().await.unwrap();
        shutdown_tx.send(()).unwrap();
        handle.await.unwrap().unwrap();
        assert_eq!(result.pid, std::process::id());
    }
}
