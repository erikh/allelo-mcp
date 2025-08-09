use super::*;
use crate::testutil::{shutdown_handle, start_api_server};

#[tokio::test]
async fn test() {
    let handle = start_api_server(Config::default()).await.unwrap();
    shutdown_handle(handle)
}
