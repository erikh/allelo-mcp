use rmcp::{
    handler::server::{router::tool::ToolRouter, tool::Parameters},
    model::*,
    service::RequestContext,
    tool, tool_handler, tool_router, RoleServer, ServerHandler,
};

#[derive(Debug, Clone, Default)]
pub struct Service {
    tool_router: ToolRouter<Self>,
}

// NOTE: these must mirror the tool list in super::tool for any effect
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
        Ok(super::tool::tool_list().into())
    }
}
