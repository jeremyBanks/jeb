use {
    super::patterns::NoeditMatcher,
    crate::{
        HookInput,
        HookInputDetails,
        HookOutput,
        HookOutputDetails,
        PermissionDecision,
    },
    eyre::{
        ContextCompat,
        Result,
    },
    std::path::Path,
};

/// Handle PreToolUse hook: block Write operations to .noedit-protected files
pub fn handle(input: &HookInput) -> Result<Option<HookOutput>> {
    // Check if this is a Write tool
    let tool_input = match &input.details {
        Some(HookInputDetails::PreToolUse {
            tool_name,
            tool_input,
            ..
        }) if tool_name == "Write" => tool_input,
        _ => return Ok(None), // Not a Write, pass through
    };

    // Extract file_path from tool_input JSON
    let file_path = tool_input
        .get("file_path")
        .and_then(|v| v.as_str())
        .context("Write tool missing file_path")?;

    // Load all .noedit files from working directory
    let cwd = Path::new(&input.cwd);
    let matcher = match NoeditMatcher::from_working_tree(cwd) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Failed to load .noedit patterns: {}", e);
            return Ok(None); // Fail open - don't block legitimate work
        }
    };

    // Check if file_path matches any .noedit pattern
    let path = Path::new(file_path);
    if matcher.matches_path(cwd, path) {
        eprintln!("Blocking write to .noedit-protected file: {}", file_path);

        // Deny the tool use
        return Ok(Some(HookOutput {
            should_continue: None,
            stop_reason: None,
            suppress_output: None,
            system_message: Some(format!(
                "Cannot write to {}: file is protected by .noedit",
                file_path
            )),
            permission_decision: Some(PermissionDecision::Deny),
            hook_specific_output: Some(HookOutputDetails::PreToolUse {
                permission_decision: Some(PermissionDecision::Deny),
                permission_decision_reason: Some(format!(
                    "File {} is protected by .noedit patterns",
                    file_path
                )),
                updated_input: None,
            }),
        }));
    }

    Ok(None) // Pass through
}
