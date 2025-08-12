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
            prompt: Some("hello, world".into()),
        })
        .await
        .unwrap();

    let x = r.recv().await.unwrap().unwrap();
    assert!(matches!(x, Event::Open));

    let x = r.recv().await.unwrap().unwrap();
    assert!(matches!(x, Event::Message(_)));

    let mut id: uuid::Uuid = Default::default();

    if let Event::Message(m) = x {
        let obj: PromptResponse = serde_json::from_str(&m.data).unwrap();
        assert!(matches!(obj, PromptResponse::Connection(_)));
        if let PromptResponse::Connection(i) = obj {
            id = i
        }
    }

    let mut i = 0;

    while let Some(Ok(m)) = r.recv().await {
        if i == 10 {
            break;
        }

        match m {
            Event::Open => {}
            Event::Message(m) => {
                let obj: PromptResponse = serde_json::from_str(&m.data).unwrap();
                eprintln!("{}", m.data);
                assert!(matches!(obj, PromptResponse::PromptResponse(_)));
                i += 1;
            }
        }
    }

    let mut r = client
        .prompt(Prompt {
            connection_id: Some(id),
            prompt: None,
        })
        .await
        .unwrap();

    let x = r.recv().await.unwrap().unwrap();
    assert!(matches!(x, Event::Open));

    let x = r.recv().await.unwrap().unwrap();
    assert!(matches!(x, Event::Message(_)));

    if let Event::Message(m) = x {
        let obj: PromptResponse = serde_json::from_str(&m.data).unwrap();
        assert!(matches!(obj, PromptResponse::Connection(_)));
    }

    let mut i = 0;

    while let Some(Ok(m)) = r.recv().await {
        if i == 10 {
            break;
        }

        match m {
            Event::Open => {}
            Event::Message(m) => {
                let obj: PromptResponse = serde_json::from_str(&m.data).unwrap();
                eprintln!("{}", m.data);
                assert!(matches!(obj, PromptResponse::PromptResponse(_)));
                i += 1;
            }
        }
    }

    assert_eq!(i, 10);
    shutdown_handle(handle);
}
