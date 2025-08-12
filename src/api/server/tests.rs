/*
use super::*;
use crate::testutil::{default_api_url, shutdown_handle, start_api_server};
use reqwest_eventsource::Event;

#[tokio::test]
async fn test_sse() {
    let handle = start_api_server(Config::default()).await.unwrap();
    let client = super::super::client::Client::new(default_api_url());
    let mut r = client
        .prompt(Prompt {
            connection_id: Default::default(),
            prompt: "hello, world".into(),
        })
        .await
        .unwrap();

    let mut i = 0;

    while let Some(Ok(m)) = r.recv().await {
        if i == 10 {
            break;
        }
        match m {
            Event::Open => {}
            Event::Message(m) => {
                i += 1;
                eprintln!("{}", m.data);
            }
        }
    }

    assert_eq!(i, 10);

    shutdown_handle(handle);
}
*/
