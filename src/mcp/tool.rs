#![allow(dead_code)]
use rmcp::model::{ListPromptsResult, Prompt, PromptArgument};

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
