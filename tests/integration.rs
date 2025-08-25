use allelo_mcp::api::client::Client;
use allelo_mcp::api::llm::*;
use allelo_mcp::api::server::{Config, LogLevel, Prompt, PromptResponse};
use allelo_mcp::testutil::*;
use reqwest_eventsource::Event;

#[tokio::test]
async fn test_llm_client() {
    async fn run_prompt(prompt: &str) {
        let client = LLMClient::new(
            LLMClientType::OllamaVicuna,
            LLMClientParams {
                base_url: "http://localhost:11434".into(),
                api_key: None,
                timeout: None,
            },
        )
        .unwrap();
        let mut response = client.prompt(prompt.into()).await.unwrap();
        while let Some(response) = response.recv().await {
            eprintln!("response from '{}' LLM client test: '{}'", prompt, response);
            assert_ne!(response, "");
        }
    }

    run_prompt("hello").await;
    run_prompt("what is two plus two?").await;
    run_prompt("what is the capital of turkey?").await;
}

#[tokio::test]
async fn test_real_server_prompt() {
    let handle = start_api_server(Config {
        listen: "127.0.0.1:8999".parse().unwrap(),
        log_level: LogLevel::Info,
        client_type: Some(LLMClientType::OllamaVicuna),
        client_params: Some(LLMClientParams {
            base_url: "http://localhost:11434".into(),
            api_key: None,
            timeout: None,
        }),
    })
    .await
    .unwrap();

    async fn run_prompt(prompt: &str) {
        let client = Client::new(default_api_url()).await.unwrap();
        let mut r = client
            .prompt(Prompt {
                connection_id: Default::default(),
                prompt: Some(prompt.into()),
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

        if let Event::Message(m) = r.recv().await.unwrap().unwrap() {
            let obj: PromptResponse = serde_json::from_str(&m.data).unwrap();
            eprintln!("{}", m.data);
            assert!(matches!(obj, PromptResponse::PromptResponse(_)));
            i += 1;
        }

        assert!(i > 0);

        r.close();

        let mut r = client
            .prompt(Prompt {
                connection_id: Some(id),
                prompt: Some(prompt.into()),
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
            if let PromptResponse::Connection(conn_id) = obj {
                assert_eq!(id, conn_id);
            }
        }

        let mut i = 0;

        // NOTE: I guess this can fail if we only get one stream response from the LLM, but this is
        // nearly impossible in my experience.
        if let Event::Message(m) = r.recv().await.unwrap().unwrap() {
            let obj: PromptResponse = serde_json::from_str(&m.data).unwrap();
            eprintln!("{}", m.data);
            assert!(matches!(obj, PromptResponse::PromptResponse(_)));
            i += 1;
        }

        assert!(i > 0);
    }

    run_prompt("hello").await;
    run_prompt("what is two plus two?").await;
    run_prompt("what is the capital of turkey?").await;

    shutdown_handle(handle);
}
