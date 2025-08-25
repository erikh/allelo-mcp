use allelo_mcp::api::llm::*;
use allelo_mcp::api::server::{Config, LogLevel};
use allelo_mcp::testutil::*;

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
    run_prompt("what was the capital of persia?").await;
    run_prompt("what was archimedes famous for doing?").await;
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

    shutdown_handle(handle);
}
