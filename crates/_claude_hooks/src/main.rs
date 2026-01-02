mod noedit;

fn main() {
    let json: serde_json::Value = serde_json::from_reader(std::io::stdin()).unwrap();
    let input: HookInput = serde_json::from_value(json.clone()).unwrap();

    // Handle different hook events
    let output = match &input.details {
        Some(HookInputDetails::SessionStart { .. }) => {
            // Existing session ID logic
            handle_session_start(&input);

            // New .noedit initialization
            if let Err(e) = noedit::session::handle(&input) {
                eprintln!("noedit SessionStart error: {}", e);
            }
            None
        }
        Some(HookInputDetails::PreToolUse { .. }) => {
            match noedit::pretooluse::handle(&input) {
                Ok(result) => result,
                Err(e) => {
                    eprintln!("noedit PreToolUse error: {}", e);
                    None // Fail open
                }
            }
        }
        Some(HookInputDetails::Stop { .. }) | Some(HookInputDetails::SubagentStop { .. }) => {
            eprintln!(">>> Stop/SubagentStop hook triggered");
            match noedit::validation::handle(&input) {
                Ok(result) => result,
                Err(e) => {
                    eprintln!("noedit Stop error: {}", e);
                    // Return error as stop reason
                    Some(HookOutput {
                        should_continue: Some(false),
                        stop_reason: Some(format!("noedit validation failed: {}", e)),
                        suppress_output: None,
                        system_message: Some(format!("Error validating .noedit: {}", e)),
                        permission_decision: None,
                        hook_specific_output: None,
                    })
                }
            }
        }
        Some(HookInputDetails::SessionEnd { .. }) => {
            eprintln!(">>> SessionEnd hook triggered");
            match noedit::validation::handle(&input) {
                Ok(result) => result,
                Err(e) => {
                    eprintln!("noedit SessionEnd error: {}", e);
                    // Don't block session end, just warn
                    Some(HookOutput {
                        should_continue: None,
                        stop_reason: None,
                        suppress_output: None,
                        system_message: Some(format!("Warning: .noedit validation failed: {}", e)),
                        permission_decision: None,
                        hook_specific_output: None,
                    })
                }
            }
        }
        _ => None,
    };

    // Output the hook result if any
    if let Some(output) = output {
        serde_json::to_writer(std::io::stdout(), &output).unwrap();
    }
}

fn handle_session_start(input: &HookInput) {
    let claude_env_file_path = std::env::var("CLAUDE_ENV_FILE").ok();
    if let Some(claude_env_file_path) = claude_env_file_path {
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(claude_env_file_path)
            .unwrap();
        writeln!(file, "JEB_CLAUDE_SESSION_ID={}", input.session_id).unwrap();
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub struct HookInput {
    pub session_id: String,
    pub transcript_path: String,
    pub cwd: String,
    pub permission_mode: Option<PermissionMode>,
    #[serde(flatten)]
    pub details: Option<HookInputDetails>,
}
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(
    deny_unknown_fields,
    tag = "hook_event_name",
    rename_all = "PascalCase"
)]
pub enum HookInputDetails {
    Notification {
        message: String,
        notification_type: String,
    },
    PreToolUse {
        tool_name: String,
        tool_input: serde_json::Value,
        tool_use_id: String,
    },
    PostToolUse {
        tool_name: String,
        tool_input: serde_json::Value,
        tool_response: serde_json::Value,
        tool_use_id: String,
    },
    PreCompactInput {
        trigger: String,
        custom_instructions: String,
    },
    SessionEnd {
        reason: String,
    },
    SessionStart {
        source: String,
    },
    Stop {
        stop_hook_active: bool,
    },
    SubagentStop {
        stop_hook_active: bool,
    },
    UserPromptSubmit {
        prompt: String,
    },
}
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(
    deny_unknown_fields,
    rename_all = "camelCase"
)]
pub struct HookOutput {
    #[serde(rename = "continue", skip_serializing_if = "Option::is_none")]
    pub should_continue: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suppress_output: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_decision: Option<PermissionDecision>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hook_specific_output: Option<HookOutputDetails>,
}
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(
    deny_unknown_fields,
    tag = "hookEventName",
    rename_all = "PascalCase",
    rename_all_fields = "camelCase"
)]
pub enum HookOutputDetails {
    Notification {
        #[serde(flatten)]
        other: serde_json::Value,
    },
    PreToolUse {
        #[serde(skip_serializing_if = "Option::is_none")]
        permission_decision: Option<PermissionDecision>,
        #[serde(skip_serializing_if = "Option::is_none")]
        permission_decision_reason: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        updated_input: Option<String>,
    },
    PostToolUse {
        additional_context: String,
    },
    PreCompactInput {
        #[serde(flatten)]
        other: serde_json::Value,
    },
    SessionEnd {
        #[serde(flatten)]
        other: serde_json::Value,
    },
    SessionStart {
        #[serde(flatten)]
        other: serde_json::Value,
    },
    Stop {
        #[serde(flatten)]
        other: serde_json::Value,
    },
    SubagentStop {
        #[serde(flatten)]
        other: serde_json::Value,
    },
    UserPromptSubmit {
        additional_context: String,
    },
}
#[derive(
    serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
#[serde(
    deny_unknown_fields,
    rename_all = "camelCase"
)]
pub enum PermissionMode {
    Plan,
    Default,
    AcceptEdits,
    BypassPermissions,
}
#[derive(
    serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
#[serde(rename_all = "camelCase")]
pub enum PermissionDecision {
    Allow,
    Deny,
    Ask,
}
