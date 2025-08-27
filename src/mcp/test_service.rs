#![allow(dead_code)]

use rmcp::{
	RoleServer, ServerHandler,
	handler::server::{router::tool::ToolRouter, tool::Parameters},
	model::*,
	service::RequestContext,
	tool, tool_handler, tool_router,
};

use crate::mcp::tool::{ToolArgument, ToolFunction, ToolList};

#[derive(Debug, Clone, Default)]
pub struct TestService {
	tool_router: ToolRouter<Self>,
}

#[tool_router]
impl TestService {
	#[tool(description = "unit test for tools")]
	pub(crate) fn test_tool(&self) -> String {
		"test passed".into()
	}

	#[tool(description = "unit test for tools with parameters")]
	pub(crate) fn test_tool_with_parameters(
		&self, Parameters(name): Parameters<String>,
	) -> String {
		format!("got parameter '{}'", name)
	}
}

#[tool_handler]
impl ServerHandler for TestService {
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
		&self, _request: Option<PaginatedRequestParam>,
		_: RequestContext<RoleServer>,
	) -> Result<ListPromptsResult, rmcp::ErrorData> {
		Ok(test_tool_list().into())
	}
}

pub(crate) fn test_tool_list() -> ToolList {
	ToolList(vec![
		ToolFunction {
			name: "test_tool".into(),
			description: "unit test for tools".into(),
			args: Default::default(),
		},
		ToolFunction {
			name: "test_tool_with_parameters".into(),
			description: "unit test for tools with parameters".into(),
			args: vec![ToolArgument {
				name: "name".to_string(),
				description: "The name of the contact or friend"
					.to_string(),
				required: true,
			}],
		},
	])
}
