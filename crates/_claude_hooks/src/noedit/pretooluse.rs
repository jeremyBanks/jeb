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

/// Handle PreToolUse hook: block Write and Edit operations to .noedit-protected
/// files
pub fn handle(input: &HookInput) -> Result<Option<HookOutput>> {
    // Check if this is a Write or Edit tool
    let (tool_name, tool_input) = match &input.details {
        Some(HookInputDetails::PreToolUse {
            tool_name,
            tool_input,
            ..
        }) if tool_name == "Write" || tool_name == "Edit" => (tool_name.as_str(), tool_input),
        _ => return Ok(None), // Not a Write or Edit, pass through
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
        eprintln!(
            "Blocking {} to .noedit-protected file: {}",
            tool_name, file_path
        );

        // Deny the tool use - minimal output with only hookSpecificOutput fields
        return Ok(Some(HookOutput {
            should_continue: None,
            stop_reason: None,
            suppress_output: None,
            system_message: None,
            permission_decision: None,
            hook_specific_output: Some(HookOutputDetails::PreToolUse {
                permission_decision: Some(PermissionDecision::Deny),
                permission_decision_reason: Some(format!(
                    "CRITICAL: AI agents MUST NOT attempt to {} this file.\n\nPath: {}\n\nThis \
                     file is protected by .noedit patterns. Any changes made to this path will be \
                     automatically reverted, which may result in broken code or lost work.\n\nIf \
                     you need to modify files in this area, please inform the user that these \
                     files are read-only and ask them to make the changes manually.",
                    tool_name, file_path
                )),
                updated_input: None,
            }),
        }));
    }

    Ok(None) // Pass through
}
