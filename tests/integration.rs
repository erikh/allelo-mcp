use allelo_mcp::api::llm::*;

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
