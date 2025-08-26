#![allow(dead_code)]
use rmcp::model::{ListPromptsResult, Prompt, PromptArgument};

#[derive(Debug, Clone)]
pub(crate) struct ToolFunction {
    name: String,
    description: String,
    args: Vec<ToolArgument>,
}

#[derive(Debug, Clone)]
pub(crate) struct ToolArgument {
    name: String,
    description: String,
    required: bool,
}

impl ToolFunction {
    pub(crate) fn into_rmcp_arguments(&self) -> Option<Vec<PromptArgument>> {
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

impl Into<PromptArgument> for ToolArgument {
    fn into(self) -> PromptArgument {
        PromptArgument {
            name: self.name,
            description: Some(self.description),
            required: Some(self.required),
        }
    }
}

pub(crate) fn tool_list() -> ListPromptsResult {
    ListPromptsResult {
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
    }
}
