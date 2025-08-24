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

    let response = client.prompt("hello".into()).await.unwrap();
    eprintln!("response from 'hello' LLM client test: '{}'", response);
    assert_ne!(response, "");
}
