use super::client::halo::Client;
use rmcp::{model::*, service::RequestContext, tool, RoleServer, ServerHandler};

#[derive(Debug, Clone, Default)]
pub(crate) struct Service {}

#[tool(tool_box)]
impl Service {
    #[tool(
        description = "collect the descriptions of all the faults, which are halo's name for issues or tickets. These will be presented as plain english"
    )]
    pub(crate) fn collect_faults(&self) -> Result<CallToolResult, rmcp::Error> {
        let client = Client::default();
        let faults = client.list_faults().map_err(Into::into)?;
        let mut result = Vec::new();
        for fault in faults {
            result.push(Content::text(fault.summary()))
        }
        Ok(CallToolResult::success(result))
    }
}

#[tool(tool_box)]
impl ServerHandler for Service {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_prompts()
                .enable_resources()
                .enable_tools()
                .build(),
            server_info: Implementation::from_build_env(),
            instructions: Some("This server provides data on halo, a ticketing system".to_string()),
        }
    }

    async fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParam>,
        _: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, rmcp::Error> {
        Ok(ListPromptsResult {
            next_cursor: None,
            prompts: vec![Prompt::new(
                "collect_faults",
                Some("Collect the description of all faults in halo"),
                Some(vec![PromptArgument {
                    name: "message".to_string(),
                    description: Some("A message to put in the prompt".to_string()),
                    required: Some(true),
                }]),
            )],
        })
    }
}
