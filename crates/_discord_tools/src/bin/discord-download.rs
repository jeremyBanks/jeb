use std::path::PathBuf;
use std::time::Duration;

use chrono::{DateTime, Utc};
use clap::Parser;
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};
use reqwest::Client;

use _discord_tools::*;

/// Download all of your messages from a Discord channel (or all text channels
/// in a server), including attachments.
///
/// Requires a Discord user token set via --token or DISCORD_TOKEN env var.
///
/// Output is a directory containing a `messages.json` file (one JSON array of
/// SavedMessage objects) and an `attachments/` subdirectory with downloaded
/// files. The messages.json file can be reviewed and annotated with
/// `"annotation": "[DELETE]"` on messages you want to delete, then fed to
/// `discord-delete`.
#[derive(Parser)]
struct Args {
    /// Discord user token (or set DISCORD_TOKEN env var).
    #[clap(long, env = "DISCORD_TOKEN")]
    token: String,

    /// Channel ID to download from. If not provided, --guild must be set and
    /// all text channels in the guild will be downloaded.
    #[clap(long)]
    channel: Option<String>,

    /// Guild (server) ID. If set without --channel, downloads from all text
    /// channels. If set with --channel, used for directory naming.
    #[clap(long)]
    guild: Option<String>,

    /// Your Discord user ID, to filter only your messages.
    #[clap(long, env = "DISCORD_USER_ID")]
    user_id: String,

    /// Output base directory.
    #[clap(long, default_value = "discord-export")]
    output: PathBuf,

    /// Also download message attachments.
    #[clap(long)]
    attachments: bool,
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

    // Determine which channels to process.
    let channels: Vec<(String, Option<String>, String)> = if let Some(channel_id) = &args.channel {
        // Single channel mode.
        let channel_name = get_channel_name(&client, channel_id).await?;
        let guild_name = if let Some(guild_id) = &args.guild {
            Some(get_guild_name(&client, guild_id).await?)
        } else {
            None
        };
        vec![(channel_id.clone(), guild_name, channel_name)]
    } else if let Some(guild_id) = &args.guild {
        // All text channels in guild.
        let guild_name = get_guild_name(&client, guild_id).await?;
        let channels = get_guild_text_channels(&client, guild_id).await?;
        eprintln!("Found {} text channels in {}", channels.len(), guild_name);
        channels
            .into_iter()
            .map(|(id, name)| (id, Some(guild_name.clone()), name))
            .collect()
    } else {
        eprintln!("Error: must provide --channel, --guild, or both.");
        std::process::exit(1);
    };

    for (channel_id, guild_name, channel_name) in &channels {
        eprintln!(
            "\n--- Downloading from #{} (channel {}) ---",
            channel_name, channel_id
        );

        let out = output_dir(&args.output, guild_name.as_deref(), channel_name);
        std::fs::create_dir_all(&out)?;

        let messages = download_channel_messages(&client, channel_id, &args.user_id).await?;
        eprintln!("  Found {} messages from you", messages.len());

        // Download attachments if requested.
        let messages = if args.attachments {
            download_attachments(&client, &out, messages).await?
        } else {
            messages
        };

        // Write messages.json
        let path = messages_file(&out);
        let json = serde_json::to_string_pretty(&messages)?;
        std::fs::write(&path, &json)?;
        eprintln!("  Saved to {}", path.display());
    }

    eprintln!("\nDone! Review the messages.json files and add `\"annotation\": \"[DELETE]\"`");
    eprintln!("to any messages you want to delete, then run discord-delete.");

    Ok(())
}

async fn get_channel_name(
    client: &Client,
    channel_id: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let url = format!("{DISCORD_API}/channels/{channel_id}");
    let resp: api::Channel = client.get(&url).send().await?.error_for_status()?.json().await?;
    Ok(resp.name.unwrap_or_else(|| format!("channel-{channel_id}")))
}

async fn get_guild_name(
    client: &Client,
    guild_id: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let url = format!("{DISCORD_API}/guilds/{guild_id}");
    let resp: api::Guild = client.get(&url).send().await?.error_for_status()?.json().await?;
    Ok(resp.name)
}

async fn get_guild_text_channels(
    client: &Client,
    guild_id: &str,
) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
    let url = format!("{DISCORD_API}/guilds/{guild_id}/channels");
    let channels: Vec<api::Channel> =
        client.get(&url).send().await?.error_for_status()?.json().await?;
    Ok(channels
        .into_iter()
        .filter(|c| c.channel_type == 0) // text channels only
        .map(|c| {
            let name = c.name.unwrap_or_else(|| format!("channel-{}", c.id));
            (c.id, name)
        })
        .collect())
}

/// Fetch all messages by a given user from a channel, paginating through
/// Discord's API (100 messages per request, using `before` cursor).
async fn download_channel_messages(
    client: &Client,
    channel_id: &str,
    user_id: &str,
) -> Result<Vec<SavedMessage>, Box<dyn std::error::Error>> {
    let mut all_messages = Vec::new();
    let mut before: Option<String> = None;

    loop {
        let mut url = format!("{DISCORD_API}/channels/{channel_id}/messages?limit=100");
        if let Some(ref b) = before {
            url.push_str(&format!("&before={b}"));
        }

        let resp = client.get(&url).send().await?;

        if resp.status().as_u16() == 429 {
            // Rate limited - wait and retry.
            let retry_after: f64 = resp
                .json::<serde_json::Value>()
                .await?
                .get("retry_after")
                .and_then(|v| v.as_f64())
                .unwrap_or(5.0);
            eprintln!("  Rate limited, waiting {:.1}s...", retry_after);
            tokio::time::sleep(Duration::from_secs_f64(retry_after + 0.5)).await;
            continue;
        }

        let resp = resp.error_for_status()?;
        let batch: Vec<api::Message> = resp.json().await?;

        if batch.is_empty() {
            break;
        }

        // Track the oldest message ID for pagination.
        before = batch.last().map(|m| m.id.clone());

        // Filter to only our user's messages.
        let our_messages: Vec<SavedMessage> = batch
            .into_iter()
            .filter(|m| m.author.id == user_id)
            .map(|m| {
                let timestamp = m
                    .timestamp
                    .parse::<DateTime<Utc>>()
                    .unwrap_or_else(|_| Utc::now());
                SavedMessage {
                    id: m.id,
                    channel_id: m.channel_id,
                    guild_id: m.guild_id,
                    author_id: m.author.id,
                    author_username: m.author.username,
                    content: m.content,
                    timestamp,
                    attachments: m
                        .attachments
                        .into_iter()
                        .map(|a| SavedAttachment {
                            id: a.id,
                            filename: a.filename,
                            url: a.url,
                            size: a.size,
                            local_path: None,
                        })
                        .collect(),
                    annotation: None,
                }
            })
            .collect();

        let batch_count = our_messages.len();
        all_messages.extend(our_messages);

        eprint!("\r  Downloaded {} messages so far...", all_messages.len());

        // If we got fewer than 100 messages, we've reached the end.
        if batch_count == 0 && before.is_some() {
            // The batch had messages but none were ours - keep going.
        }
        // Small delay to be respectful of rate limits.
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    eprintln!();

    // Sort by timestamp (oldest first) for readable output.
    all_messages.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

    Ok(all_messages)
}

/// Download attachment files for messages, saving them into an attachments/
/// subdirectory and updating the local_path field.
async fn download_attachments(
    client: &Client,
    out_dir: &std::path::Path,
    mut messages: Vec<SavedMessage>,
) -> Result<Vec<SavedMessage>, Box<dyn std::error::Error>> {
    let att_dir = attachments_dir(out_dir);
    std::fs::create_dir_all(&att_dir)?;

    let mut count = 0u64;
    for msg in &mut messages {
        for att in &mut msg.attachments {
            // Use message_id-attachment_id-filename to avoid collisions.
            let local_name = format!("{}-{}-{}", msg.id, att.id, att.filename);
            let local_path = att_dir.join(&local_name);

            if local_path.exists() {
                att.local_path = Some(format!("attachments/{local_name}"));
                continue;
            }

            match client.get(&att.url).send().await {
                Ok(resp) => match resp.error_for_status() {
                    Ok(resp) => {
                        let bytes = resp.bytes().await?;
                        std::fs::write(&local_path, &bytes)?;
                        att.local_path = Some(format!("attachments/{local_name}"));
                        count += 1;
                    }
                    Err(e) => {
                        eprintln!("  Warning: failed to download attachment {}: {e}", att.filename);
                    }
                },
                Err(e) => {
                    eprintln!("  Warning: failed to download attachment {}: {e}", att.filename);
                }
            }

            tokio::time::sleep(Duration::from_millis(200)).await;
        }
    }

    if count > 0 {
        eprintln!("  Downloaded {count} attachments");
    }

    Ok(messages)
}
