#![allow(dead_code)]
use llm::{
	builder::{FunctionBuilder, ParamBuilder},
	chat::{FunctionTool, Tool},
};
use rmcp::model::{ListPromptsResult, Prompt, PromptArgument};
use serde::Serialize;

// NOTE: rmcp and llm are server and client implementations of MCP respectively, but they use
// independent types. Most of the fields are very similar, and the serialized result is exactly the
// same, so a lot of this is from/into to ensure these types translate between each other cleanly.

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ToolList(pub(crate) Vec<ToolFunction>);

impl Into<ListPromptsResult> for ToolList {
	fn into(self) -> ListPromptsResult {
		ListPromptsResult {
			next_cursor: None,
			prompts: self.0.iter().map(|x| x.clone().into()).collect(),
		}
	}
}

impl Into<Vec<Tool>> for ToolList {
	fn into(self) -> Vec<Tool> {
		self.0.iter().map(|x| x.clone().into()).collect()
	}
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ToolFunction {
	pub(crate) name: String,
	pub(crate) description: String,
	pub(crate) args: Vec<ToolArgument>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ToolArgument {
	pub(crate) name: String,
	pub(crate) description: String,
	pub(crate) required: bool,
}

impl ToolFunction {
	pub(crate) fn required_arguments(&self) -> Vec<String> {
		let mut v = Vec::new();

		for item in &self.args {
			if item.required {
				v.push(item.name.clone())
			}
		}

		v
	}

	pub(crate) fn into_rmcp_arguments(
		&self,
	) -> Option<Vec<PromptArgument>> {
		if self.args.is_empty() {
			None
		} else {
			Some(self.args.iter().map(|x| x.clone().into()).collect())
		}
	}
}

impl Into<Prompt> for ToolFunction {
	fn into(self) -> Prompt {
		Prompt::new(
			&self.name,
			Some(&self.description),
			self.into_rmcp_arguments(),
		)
	}
}

impl Into<Tool> for ToolFunction {
	fn into(self) -> Tool {
		Tool {
			function: FunctionTool {
				name: self.name,
				description: self.description,
				parameters: self
					.args
					.iter()
					.map(|x| Into::<serde_json::Value>::into(x.clone()))
					.collect(),
			},
			tool_type: "".into(),
		}
	}
}

impl Into<serde_json::Value> for ToolArgument {
	fn into(self) -> serde_json::Value {
		serde_json::to_value(self).unwrap()
	}
}

impl Into<ParamBuilder> for ToolArgument {
	fn into(self) -> ParamBuilder {
		ParamBuilder::new(&self.name).description(&self.description)
	}
}

impl Into<FunctionBuilder> for ToolFunction {
	fn into(self) -> FunctionBuilder {
		let mut builder = FunctionBuilder::new(&self.name)
			.required(self.required_arguments())
			.description(&self.description);

		for arg in self.args {
			builder = builder.param(arg.into())
		}

		builder
	}
}

impl Into<PromptArgument> for ToolArgument {
	fn into(self) -> PromptArgument {
		PromptArgument {
			name: self.name,
			description: Some(self.description),
			required: Some(self.required),
		}
	}
}

pub(crate) fn tool_list() -> ToolList {
	ToolList(vec![
        ToolFunction {
            name: "all_contacts".into(),
            description: "list of all contacts or friends".into(),
            args: Default::default(),
        },
        ToolFunction {
            name: "contact_info".into(),
            description: "information on a specific contact or friend".into(),
            args: vec![ToolArgument {
                name: "name".to_string(),
                description: "The name of the contact or friend".to_string(),
                required: true,
            }],
        },
        ToolFunction {
            name: "contact_network".into(),
            description: "information about the friends or contacts of another contact or friend"
                .into(),
            args: vec![ToolArgument {
                name: "name".to_string(),
                description: "The name of the contact or friend".to_string(),
                required: true,
            }],
        },
        ToolFunction {
            name: "chat_messages".into(),
            description: "recent chat messages with a friend or contact".into(),
            args: vec![ToolArgument {
                name: "name".to_string(),
                description: "The name of the contact or friend".to_string(),
                required: true,
            }],
        },
        ToolFunction {
            name: "group_chat".into(),
            description: "recent messages inside a group chat".into(),
            args: vec![ToolArgument {
                name: "name".to_string(),
                description: "The name of the group".to_string(),
                required: true,
            }],
        },
        ToolFunction {
            name: "contact_activity".into(),
            description: "online activity information about a friend or contact".into(),
            args: vec![ToolArgument {
                name: "name".to_string(),
                description: "The name of the contact or friend".to_string(),
                required: true,
            }],
        },
        ToolFunction {
            name: "contact_status".into(),
            description: "status information about a friend or contact".into(),
            args: vec![ToolArgument {
                name: "name".to_string(),
                description: "The name of the contact or friend".to_string(),
                required: true,
            }],
        },
    ])
}
