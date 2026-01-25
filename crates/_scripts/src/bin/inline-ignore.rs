use {
    anyhow::{Context, Result, bail},
    std::{
        collections::HashMap,
        env,
        fs,
        path::{Path, PathBuf},
    },
};

/// A line from a gitignore file with its original content
#[derive(Debug, Clone)]
struct SourceLine {
    content: String,
}

/// A group of lines: optional preamble (comments/blanks) followed by optional pattern
#[derive(Debug, Clone)]
struct LineGroup {
    /// Consecutive blank lines (collapsed to one entry) or comment lines
    preamble: Vec<SourceLine>,
    /// The actual pattern line (None if file ends with comments/blanks)
    pattern: Option<SourceLine>,
}

/// Result of transforming a pattern
#[derive(Debug, Clone)]
struct TransformResult {
    /// The transformed pattern(s)
    patterns: Vec<String>,
    /// Whether this pattern exclusively applies to the target (can be pruned from ancestor)
    is_exclusive: bool,
}

/// Classification of a gitignore pattern
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PatternKind {
    /// No `/` except possibly trailing (e.g., `foo.txt`, `*.log`, `build/`)
    Simple,
    /// Has `/` in middle or leading `/` (e.g., `/foo`, `a/b/foo`)
    Anchored,
    /// Starts with `**/` (e.g., `**/foo.txt`, `**/a/b/foo`)
    DoubleStar,
}

/// Classify a gitignore pattern (after stripping negation and trailing slash)
fn classify_pattern(pattern: &str) -> PatternKind {
    // Strip negation prefix for classification
    let pattern = pattern.strip_prefix('!').unwrap_or(pattern);

    // Strip trailing slash for classification
    let pattern = pattern.strip_suffix('/').unwrap_or(pattern);

    if pattern.starts_with("**/") {
        return PatternKind::DoubleStar;
    }

    // Check for slash in the pattern (leading or middle)
    if pattern.contains('/') {
        return PatternKind::Anchored;
    }

    PatternKind::Simple
}

/// Check if a line is a comment
fn is_comment(line: &str) -> bool {
    line.trim_start().starts_with('#')
}

/// Check if a line is blank (empty or whitespace only)
fn is_blank(line: &str) -> bool {
    line.trim().is_empty()
}

/// Check if a pattern path prefix contains wildcards
fn has_wildcard_in_prefix(pattern_parts: &[&str], prefix_len: usize) -> bool {
    pattern_parts[..prefix_len].iter().any(|part| {
        part.contains('*') || part.contains('?') || part.contains('[')
    })
}

/// Parse a gitignore file into line groups
fn parse_gitignore(content: &str) -> Vec<LineGroup> {
    let mut groups = Vec::new();
    let mut current_preamble: Vec<SourceLine> = Vec::new();
    let mut in_blank_run = false;

    let lines: Vec<&str> = content.lines().collect();

    // Trim trailing blank lines (we ignore them per spec)
    let mut end = lines.len();
    while end > 0 && is_blank(lines[end - 1]) {
        end -= 1;
    }
    let lines = &lines[..end];

    for line in lines {
        if is_blank(line) {
            if !in_blank_run {
                // Start of a new blank run - add as single entry
                current_preamble.push(SourceLine { content: line.to_string() });
                in_blank_run = true;
            }
            // If already in a blank run, skip (multiple blanks = single entry)
        } else if is_comment(line) {
            in_blank_run = false;
            current_preamble.push(SourceLine { content: line.to_string() });
        } else {
            // Pattern line
            in_blank_run = false;
            groups.push(LineGroup {
                preamble: std::mem::take(&mut current_preamble),
                pattern: Some(SourceLine { content: line.to_string() }),
            });
        }
    }

    // If there's leftover preamble with no pattern, add it as a group
    if !current_preamble.is_empty() {
        groups.push(LineGroup {
            preamble: current_preamble,
            pattern: None,
        });
    }

    groups
}

/// Transform a pattern for use in a child gitignore
/// Returns None if pattern doesn't apply, Some(TransformResult) with transformed pattern(s)
fn transform_pattern(pattern: &str, relative_path: &[String]) -> Option<TransformResult> {
    if relative_path.is_empty() {
        // Same directory, no transformation needed
        return Some(TransformResult {
            patterns: vec![pattern.to_string()],
            is_exclusive: false,
        });
    }

    // Handle negation
    let (negation, pattern) = if let Some(p) = pattern.strip_prefix('!') {
        ("!", p)
    } else {
        ("", pattern)
    };

    // Handle trailing slash
    let (pattern, trailing_slash) = if let Some(p) = pattern.strip_suffix('/') {
        (p, "/")
    } else {
        (pattern, "")
    };

    let kind = classify_pattern(pattern);

    let (results, is_exclusive) = match kind {
        PatternKind::Simple => {
            // Copy as-is, not exclusive (matches at any level)
            (vec![pattern.to_string()], false)
        }
        PatternKind::Anchored => {
            let (transformed, exclusive) = transform_anchored(pattern, relative_path)?;
            (transformed, exclusive)
        }
        PatternKind::DoubleStar => {
            // Never exclusive (matches at multiple levels due to **/)
            (transform_double_star(pattern, relative_path), false)
        }
    };

    // Re-add negation and trailing slash
    let results: Vec<String> = results
        .into_iter()
        .map(|p| format!("{}{}{}", negation, p, trailing_slash))
        .collect();

    Some(TransformResult {
        patterns: results,
        is_exclusive,
    })
}

/// Transform an anchored pattern (has `/` in middle or leading `/`)
/// Returns (transformed_patterns, is_exclusive)
fn transform_anchored(pattern: &str, relative_path: &[String]) -> Option<(Vec<String>, bool)> {
    let had_leading_slash = pattern.starts_with('/');
    let pattern = pattern.strip_prefix('/').unwrap_or(pattern);

    // Split pattern into components
    let pattern_parts: Vec<&str> = pattern.split('/').collect();

    // Check if pattern starts with the relative path
    if pattern_parts.len() < relative_path.len() {
        return None;
    }

    // Check for wildcards in the prefix that would match the relative path
    // If there are wildcards, we can't be sure this exclusively applies to target
    if has_wildcard_in_prefix(&pattern_parts, relative_path.len()) {
        return None;
    }

    for (i, component) in relative_path.iter().enumerate() {
        if pattern_parts.get(i) != Some(&component.as_str()) {
            return None;
        }
    }

    // Strip the relative path prefix
    let remaining: Vec<&str> = pattern_parts[relative_path.len()..].to_vec();

    if remaining.is_empty() {
        // Pattern matches exactly the target directory itself
        // This would be weird but handle it
        return None;
    }

    let result = remaining.join("/");

    // Determine if we need a leading slash
    // We need it if: result has no slash in it (would match at any level otherwise)
    // OR if original had leading slash and result still has slashes (preserve style)
    let needs_leading_slash = !result.contains('/') || had_leading_slash;

    let result = if needs_leading_slash {
        format!("/{}", result)
    } else {
        result
    };

    // This pattern exclusively applies to the target directory
    // (it's anchored and starts with the exact target path, no wildcards in prefix)
    Some((vec![result], true))
}

/// Transform a double-star pattern (`**/...`)
fn transform_double_star(pattern: &str, relative_path: &[String]) -> Vec<String> {
    // Pattern starts with **/, get the rest
    let after_stars = pattern.strip_prefix("**/").unwrap_or(pattern);

    // If no slash in the rest, just copy as-is
    if !after_stars.contains('/') {
        return vec![pattern.to_string()];
    }

    let mut results = vec![pattern.to_string()];

    // Check each suffix of relative_path to see if it's a prefix of after_stars
    let after_parts: Vec<&str> = after_stars.split('/').collect();

    for suffix_start in 0..relative_path.len() {
        let suffix: Vec<&str> = relative_path[suffix_start..]
            .iter()
            .map(|s| s.as_str())
            .collect();

        // Check if suffix is a prefix of after_parts
        if after_parts.len() >= suffix.len() {
            let matches = suffix.iter()
                .zip(after_parts.iter())
                .all(|(a, b)| a == b);

            if matches {
                // Found a match! Create anchored pattern for remainder
                let remainder: Vec<&str> = after_parts[suffix.len()..].to_vec();
                if !remainder.is_empty() {
                    let anchored = format!("/{}", remainder.join("/"));
                    if !results.contains(&anchored) {
                        results.insert(0, anchored); // Insert anchored version first
                    }
                }
            }
        }
    }

    results
}

/// Find insertion point in target for a line, given the previous line from source
fn find_insertion_point(target_lines: &[String], previous_line: Option<&str>) -> usize {
    match previous_line {
        None => {
            // First line in source file - append at end
            target_lines.len()
        }
        Some(prev) => {
            // Find the line matching previous_line
            for (i, line) in target_lines.iter().enumerate() {
                if line == prev {
                    return i + 1;
                }
            }
            // Not found - append at end
            target_lines.len()
        }
    }
}

/// Merge transformed groups into target content
fn merge_into_target(target_content: &str, groups: Vec<LineGroup>) -> String {
    let mut target_lines: Vec<String> = target_content.lines().map(|s| s.to_string()).collect();

    // Trim trailing blank lines from target
    while let Some(last) = target_lines.last() {
        if is_blank(last) {
            target_lines.pop();
        } else {
            break;
        }
    }

    let mut previous_line: Option<String> = None;

    for group in groups {
        // Determine if we should include this group's preamble
        let include_preamble = group.pattern.is_some();

        if let Some(ref pattern) = group.pattern {
            // Check if pattern already exists
            if target_lines.contains(&pattern.content) {
                previous_line = Some(pattern.content.clone());
                continue;
            }

            // Find insertion point
            let insert_at = find_insertion_point(&target_lines, previous_line.as_deref());

            // Insert preamble if applicable
            let mut inserted_count = 0;
            if include_preamble {
                for preamble_line in &group.preamble {
                    target_lines.insert(insert_at + inserted_count, preamble_line.content.clone());
                    inserted_count += 1;
                }
            }

            // Insert the pattern
            target_lines.insert(insert_at + inserted_count, pattern.content.clone());

            previous_line = Some(pattern.content.clone());
        }
    }

    // Ensure trailing blank line
    if !target_lines.is_empty() {
        target_lines.push(String::new());
    }

    target_lines.join("\n")
}

/// Remove exclusive patterns from an ancestor gitignore
fn prune_ancestor(gitignore_path: &Path, patterns_to_remove: &[String]) -> Result<String> {
    let content = fs::read_to_string(gitignore_path)
        .with_context(|| format!("Cannot read {}", gitignore_path.display()))?;

    let groups = parse_gitignore(&content);
    let mut result_lines: Vec<String> = Vec::new();
    let mut pending_preamble: Vec<String> = Vec::new();

    for group in groups {
        if let Some(ref pattern) = group.pattern {
            if patterns_to_remove.contains(&pattern.content) {
                // Skip this pattern and its preamble
                pending_preamble.clear();
                continue;
            }
            // Keep this pattern - flush any pending preamble and this group
            result_lines.extend(pending_preamble.drain(..));
            for p in &group.preamble {
                result_lines.push(p.content.clone());
            }
            result_lines.push(pattern.content.clone());
        } else {
            // Group with only preamble - hold it pending
            for p in &group.preamble {
                pending_preamble.push(p.content.clone());
            }
        }
    }

    // Don't include trailing preamble that had no following pattern

    // Ensure trailing blank line if file is not empty
    if !result_lines.is_empty() {
        result_lines.push(String::new());
    }

    Ok(result_lines.join("\n"))
}

/// Find git repository root
fn find_git_root(start: &Path) -> Result<PathBuf> {
    let mut current = start.to_path_buf();
    loop {
        if current.join(".git").exists() {
            return Ok(current);
        }
        if !current.pop() {
            bail!("Not in a git repository");
        }
    }
}

/// Collect all parent gitignore files from root to target (exclusive of target)
fn collect_parent_gitignores(git_root: &Path, target_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut gitignores = Vec::new();

    let relative = target_dir.strip_prefix(git_root)
        .context("target is not under git root")?;

    let mut current = git_root.to_path_buf();

    // Check root
    let root_gitignore = current.join(".gitignore");
    if root_gitignore.exists() {
        gitignores.push(root_gitignore);
    }

    // Walk down through intermediate directories
    for component in relative.components() {
        current = current.join(component);
        if current == target_dir {
            break;
        }
        let gitignore = current.join(".gitignore");
        if gitignore.exists() {
            gitignores.push(gitignore);
        }
    }

    Ok(gitignores)
}

/// Get relative path components from source directory to target directory
fn get_relative_path_components(source_dir: &Path, target_dir: &Path) -> Result<Vec<String>> {
    let relative = target_dir.strip_prefix(source_dir)
        .context("target is not under source")?;

    Ok(relative
        .components()
        .map(|c| c.as_os_str().to_string_lossy().to_string())
        .collect())
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    let mut dry_run = false;
    let mut prune = false;
    let mut target_path = None;

    for arg in &args[1..] {
        if arg == "--dry-run" {
            dry_run = true;
        } else if arg == "--prune" {
            prune = true;
        } else if arg.starts_with('-') {
            bail!("Unknown option: {}", arg);
        } else {
            if target_path.is_some() {
                bail!("Multiple target paths specified");
            }
            target_path = Some(arg.clone());
        }
    }

    let target_path = target_path
        .context("Usage: inline-ignore [--dry-run] [--prune] <target-path>")?;

    let target_dir = PathBuf::from(&target_path).canonicalize()
        .with_context(|| format!("Cannot resolve path: {}", target_path))?;

    if !target_dir.is_dir() {
        bail!("Target must be a directory: {}", target_dir.display());
    }

    let git_root = find_git_root(&target_dir)?;

    if target_dir == git_root {
        bail!("Target directory is the git root; no parent gitignores to inline");
    }

    let target_gitignore = target_dir.join(".gitignore");

    // Read existing target content (or empty if doesn't exist)
    let target_content = if target_gitignore.exists() {
        fs::read_to_string(&target_gitignore)
            .with_context(|| format!("Cannot read {}", target_gitignore.display()))?
    } else {
        String::new()
    };

    let mut result_content = target_content.clone();

    // Track exclusive patterns per ancestor for pruning
    let mut prune_info: HashMap<PathBuf, Vec<String>> = HashMap::new();

    // Collect and process parent gitignores (root to target order)
    let parent_gitignores = collect_parent_gitignores(&git_root, &target_dir)?;

    for gitignore_path in &parent_gitignores {
        let source_dir = gitignore_path.parent().unwrap();
        let relative_path = get_relative_path_components(source_dir, &target_dir)?;

        let source_content = fs::read_to_string(gitignore_path)
            .with_context(|| format!("Cannot read {}", gitignore_path.display()))?;

        let groups = parse_gitignore(&source_content);

        // Transform each group's pattern
        let mut transformed_groups = Vec::new();
        for group in groups {
            if let Some(ref pattern) = group.pattern {
                if let Some(result) = transform_pattern(&pattern.content, &relative_path) {
                    // Track exclusive patterns for potential pruning
                    if result.is_exclusive {
                        prune_info
                            .entry(gitignore_path.clone())
                            .or_default()
                            .push(pattern.content.clone());
                    }

                    // For each transformed pattern, create a group
                    for (i, t_pattern) in result.patterns.into_iter().enumerate() {
                        transformed_groups.push(LineGroup {
                            // Only include preamble for the first transformed pattern
                            preamble: if i == 0 { group.preamble.clone() } else { vec![] },
                            pattern: Some(SourceLine { content: t_pattern }),
                        });
                    }
                }
            } else {
                // Group with only preamble (trailing comments)
                transformed_groups.push(group);
            }
        }

        result_content = merge_into_target(&result_content, transformed_groups);
    }

    // Count total exclusive patterns
    let total_exclusive: usize = prune_info.values().map(|v| v.len()).sum();

    if dry_run {
        println!("# Would write to: {}", target_gitignore.display());
        println!("{}", result_content);

        if prune && total_exclusive > 0 {
            println!();
            println!("# Would prune from ancestors:");
            for (path, patterns) in &prune_info {
                if !patterns.is_empty() {
                    println!("# {}: {} pattern(s)", path.display(), patterns.len());
                    for p in patterns {
                        println!("#   - {}", p);
                    }
                    println!();
                    println!("# New content for {}:", path.display());
                    let pruned = prune_ancestor(path, patterns)?;
                    for line in pruned.lines() {
                        println!("# {}", line);
                    }
                    println!();
                }
            }
        } else if !prune && total_exclusive > 0 {
            // Hint that pruning is possible
            eprintln!();
            eprintln!(
                "Note: {} pattern(s) in ancestor(s) exclusively apply to this target and could be pruned with --prune:",
                total_exclusive
            );
            for (path, patterns) in &prune_info {
                if !patterns.is_empty() {
                    eprintln!("  {}: {} pattern(s)", path.display(), patterns.len());
                    for p in patterns {
                        eprintln!("    {}", p);
                    }
                }
            }
        }
    } else {
        fs::write(&target_gitignore, &result_content)
            .with_context(|| format!("Cannot write {}", target_gitignore.display()))?;
        println!("Updated: {}", target_gitignore.display());

        if prune && total_exclusive > 0 {
            for (path, patterns) in &prune_info {
                if !patterns.is_empty() {
                    let pruned = prune_ancestor(path, patterns)?;
                    fs::write(path, &pruned)
                        .with_context(|| format!("Cannot write {}", path.display()))?;
                    println!("Pruned {} pattern(s) from: {}", patterns.len(), path.display());
                }
            }
        } else if !prune && total_exclusive > 0 {
            // Hint that pruning is possible
            eprintln!();
            eprintln!(
                "Note: {} pattern(s) in ancestor(s) exclusively apply to this target and could be pruned with --prune:",
                total_exclusive
            );
            for (path, patterns) in &prune_info {
                if !patterns.is_empty() {
                    eprintln!("  {}: {} pattern(s)", path.display(), patterns.len());
                    for p in patterns {
                        eprintln!("    {}", p);
                    }
                }
            }
        }
    }

    Ok(())
}
