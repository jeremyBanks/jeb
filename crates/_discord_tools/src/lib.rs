use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A downloaded Discord message with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedMessage {
    pub id: String,
    pub channel_id: String,
    pub guild_id: Option<String>,
    pub author_id: String,
    pub author_username: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub attachments: Vec<SavedAttachment>,
    /// User annotation added during review. Set to "[DELETE]" to mark for deletion.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub annotation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedAttachment {
    pub id: String,
    pub filename: String,
    pub url: String,
    pub size: u64,
    /// Local path where the attachment was saved, relative to the output directory.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_path: Option<String>,
}

/// Discord API response types (subset of what we need).
pub mod api {
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    pub struct Message {
        pub id: String,
        pub channel_id: String,
        #[serde(default)]
        pub guild_id: Option<String>,
        pub author: User,
        pub content: String,
        pub timestamp: String,
        #[serde(default)]
        pub attachments: Vec<Attachment>,
    }

    #[derive(Debug, Deserialize)]
    pub struct User {
        pub id: String,
        pub username: String,
    }

    #[derive(Debug, Deserialize)]
    pub struct Attachment {
        pub id: String,
        pub filename: String,
        pub url: String,
        pub size: u64,
    }

    #[derive(Debug, Deserialize)]
    pub struct Channel {
        pub id: String,
        #[serde(default)]
        pub name: Option<String>,
        #[serde(default)]
        pub guild_id: Option<String>,
        /// Channel type (0 = text, 2 = voice, etc.)
        #[serde(rename = "type")]
        pub channel_type: u8,
    }

    #[derive(Debug, Deserialize)]
    pub struct Guild {
        pub id: String,
        pub name: String,
    }
}

/// Build the output directory path for a download.
pub fn output_dir(base: &Path, guild_name: Option<&str>, channel_name: &str) -> PathBuf {
    let mut path = base.to_path_buf();
    if let Some(guild) = guild_name {
        path.push(sanitize_filename(guild));
    }
    path.push(sanitize_filename(channel_name));
    path
}

/// Messages file within an output directory.
pub fn messages_file(dir: &Path) -> PathBuf {
    dir.join("messages.json")
}

/// Attachments subdirectory within an output directory.
pub fn attachments_dir(dir: &Path) -> PathBuf {
    dir.join("attachments")
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' { c } else { '_' })
        .collect()
}
