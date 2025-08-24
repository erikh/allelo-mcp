use allelo_mcp::api::llm::*;

#[tokio::test]
async fn test_llm_client() {
    let client = LLMClient::new(
        LLMClientType::OllamaVicuna,
        LLMClientParams {
            base_url: "http://localhost:11434".into(),
            api_key: None,
            timeout: None,
        },
    )
    .unwrap();

    let mut r = client.prompt("hello".into()).await;
    while let Some(response) = r.recv().await {
        eprintln!("{}", response);
    }
}
