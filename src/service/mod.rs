use rmcp::{
    handler::server::{router::tool::ToolRouter, tool::Parameters},
    model::*,
    service::RequestContext,
    tool, tool_handler, tool_router, RoleServer, ServerHandler,
};

#[derive(Debug, Clone, Default)]
pub(crate) struct Service {
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl Service {
    #[tool(description = "list of all contacts or friends")]
    pub(crate) fn all_contacts(&self) -> String {
        String::new()
    }

    #[tool(description = "information on a specific contact or friend")]
    pub(crate) fn contact_info(&self, Parameters(_name): Parameters<String>) -> String {
        String::new()
    }

    #[tool(description = "information about the friends or contacts of another contact or friend")]
    pub(crate) fn contact_network(&self, Parameters(_name): Parameters<String>) -> String {
        String::new()
    }

    #[tool(description = "recent chat messages with a friend or contact")]
    pub(crate) fn chat_messages(&self, Parameters(_name): Parameters<String>) -> String {
        String::new()
    }

    #[tool(description = "recent messages inside a group chat")]
    pub(crate) fn group_chat(&self, Parameters(_name): Parameters<String>) -> String {
        String::new()
    }

    #[tool(description = "online activity information about a friend or contact")]
    pub(crate) fn contact_activity(&self, Parameters(_name): Parameters<String>) -> String {
        String::new()
    }

    #[tool(description = "status information about a friend or contact")]
    pub(crate) fn contact_status(&self, Parameters(_name): Parameters<String>) -> String {
        String::new()
    }
}

#[tool_handler]
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
            instructions: Some(String::new()),
        }
    }

    async fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParam>,
        _: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, rmcp::ErrorData> {
        Ok(ListPromptsResult {
            next_cursor: None,
            prompts: vec![
                Prompt::new(
                    "all_contacts",
                    Some("list of all contacts or friends"),
                    None,
                ),
                Prompt::new(
                    "contact_info",
                    Some("information on a specific contact or friend"),
                    Some(vec![PromptArgument {
                        name: "name".to_string(),
                        description: Some("The name of the contact or friend".to_string()),
                        required: Some(true),
                    }]),
                ),
                Prompt::new(
                    "contact_network",
                    Some("information about the friends or contacts of another contact or friend"),
                    Some(vec![PromptArgument {
                        name: "name".to_string(),
                        description: Some("The name of the contact or friend".to_string()),
                        required: Some(true),
                    }]),
                ),
                Prompt::new(
                    "chat_messages",
                    Some("recent chat messages with a friend or contact"),
                    Some(vec![PromptArgument {
                        name: "name".to_string(),
                        description: Some("The name of the contact or friend".to_string()),
                        required: Some(true),
                    }]),
                ),
                Prompt::new(
                    "group_chat",
                    Some("recent messages inside a group chat"),
                    Some(vec![PromptArgument {
                        name: "name".to_string(),
                        description: Some("The name of the group".to_string()),
                        required: Some(true),
                    }]),
                ),
                Prompt::new(
                    "contact_activity",
                    Some("online activity information about a friend or contact"),
                    Some(vec![PromptArgument {
                        name: "name".to_string(),
                        description: Some("The name of the contact or friend".to_string()),
                        required: Some(true),
                    }]),
                ),
                Prompt::new(
                    "contact_status",
                    Some("status information about a friend or contact"),
                    Some(vec![PromptArgument {
                        name: "name".to_string(),
                        description: Some("The name of the contact or friend".to_string()),
                        required: Some(true),
                    }]),
                ),
            ],
        })
    }
}
