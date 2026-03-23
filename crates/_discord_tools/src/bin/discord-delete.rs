use std::path::PathBuf;
use std::time::Duration;

use clap::Parser;
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};
use reqwest::Client;

use _discord_tools::*;

/// Delete Discord messages that have been annotated with [DELETE].
///
/// Reads messages.json files produced by `discord-download`, finds messages
/// where `"annotation": "[DELETE]"` has been set, and deletes them via the
/// Discord API. Deleted messages are then marked with `"annotation":
/// "[DELETED]"` in the file so they won't be retried.
///
/// This is intentionally a two-step process: download first, review and
/// annotate the JSON, then run this tool. Nothing is deleted without explicit
/// annotation.
#[derive(Parser)]
struct Args {
    /// Discord user token (or set DISCORD_TOKEN env var).
    #[clap(long, env = "DISCORD_TOKEN")]
    token: String,

    /// Path(s) to messages.json files or directories containing them.
    /// If a directory is given, all messages.json files within are processed.
    #[clap(required = true)]
    paths: Vec<PathBuf>,

    /// Dry run - show what would be deleted without actually deleting.
    #[clap(long)]
    dry_run: bool,
}

const DISCORD_API: &str = "https://discord.com/api/v10";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&args.token)?);

    let client = Client::builder()
        .default_headers(headers)
        .timeout(Duration::from_secs(30))
        .build()?;

    // Collect all messages.json file paths.
    let mut json_files: Vec<PathBuf> = Vec::new();
    for path in &args.paths {
        if path.is_file() {
            json_files.push(path.clone());
        } else if path.is_dir() {
            collect_messages_files(path, &mut json_files)?;
        } else {
            eprintln!("Warning: {} does not exist, skipping", path.display());
        }
    }

    if json_files.is_empty() {
        eprintln!("No messages.json files found.");
        return Ok(());
    }

    eprintln!("Found {} messages.json file(s) to process", json_files.len());

    let mut total_deleted = 0u64;
    let mut total_failed = 0u64;

    for json_file in &json_files {
        eprintln!("\n--- Processing {} ---", json_file.display());

        let content = std::fs::read_to_string(json_file)?;
        let mut messages: Vec<SavedMessage> = serde_json::from_str(&content)?;

        let to_delete: Vec<usize> = messages
            .iter()
            .enumerate()
            .filter(|(_, m)| m.annotation.as_deref() == Some("[DELETE]"))
            .map(|(i, _)| i)
            .collect();

        if to_delete.is_empty() {
            eprintln!("  No messages annotated with [DELETE]");
            continue;
        }

        eprintln!(
            "  Found {} message(s) annotated with [DELETE]",
            to_delete.len()
        );

        if args.dry_run {
            for &idx in &to_delete {
                let msg = &messages[idx];
                eprintln!(
                    "  [DRY RUN] Would delete message {} in channel {} ({}): {:?}",
                    msg.id,
                    msg.channel_id,
                    msg.timestamp.format("%Y-%m-%d %H:%M:%S"),
                    truncate(&msg.content, 80),
                );
            }
            continue;
        }

        for &idx in &to_delete {
            let msg = &messages[idx];
            let url = format!(
                "{DISCORD_API}/channels/{}/messages/{}",
                msg.channel_id, msg.id
            );

            eprintln!(
                "  Deleting message {} ({}): {:?}",
                msg.id,
                msg.timestamp.format("%Y-%m-%d %H:%M:%S"),
                truncate(&msg.content, 60),
            );

            match delete_with_retry(&client, &url).await {
                Ok(()) => {
                    messages[idx].annotation = Some("[DELETED]".to_string());
                    total_deleted += 1;
                }
                Err(e) => {
                    eprintln!("    FAILED: {e}");
                    messages[idx].annotation = Some(format!("[DELETE FAILED: {e}]"));
                    total_failed += 1;
                }
            }

            // Respect rate limits - Discord allows ~5 deletes per 5 seconds
            // for user accounts. Be conservative.
            tokio::time::sleep(Duration::from_millis(1200)).await;
        }

        // Write back the updated file (with [DELETED] annotations).
        let updated = serde_json::to_string_pretty(&messages)?;
        std::fs::write(json_file, &updated)?;
        eprintln!("  Updated {}", json_file.display());
    }

    eprintln!("\n=== Summary ===");
    eprintln!("  Deleted: {total_deleted}");
    if total_failed > 0 {
        eprintln!("  Failed:  {total_failed}");
    }

    Ok(())
}

/// Recursively find all messages.json files under a directory.
fn collect_messages_files(
    dir: &std::path::Path,
    out: &mut Vec<PathBuf>,
) -> Result<(), Box<dyn std::error::Error>> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_messages_files(&path, out)?;
        } else if path.file_name().is_some_and(|n| n == "messages.json") {
            out.push(path);
        }
    }
    Ok(())
}

/// Delete a message, handling rate limits with retry.
async fn delete_with_retry(
    client: &Client,
    url: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let max_retries = 5;
    for attempt in 0..max_retries {
        let resp = client.delete(url).send().await?;
        let status = resp.status().as_u16();

        match status {
            204 => return Ok(()),
            404 => {
                // Already deleted.
                eprintln!("    (message already deleted)");
                return Ok(());
            }
            429 => {
                let retry_after: f64 = resp
                    .json::<serde_json::Value>()
                    .await?
                    .get("retry_after")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(5.0);
                eprintln!(
                    "    Rate limited (attempt {}/{}), waiting {:.1}s...",
                    attempt + 1,
                    max_retries,
                    retry_after
                );
                tokio::time::sleep(Duration::from_secs_f64(retry_after + 0.5)).await;
            }
            _ => {
                let body = resp.text().await.unwrap_or_default();
                return Err(format!("HTTP {status}: {body}").into());
            }
        }
    }

    Err("max retries exceeded due to rate limiting".into())
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max])
    }
}
