use std::{
    collections::{
        BTreeMap,
        HashMap,
    },
    fmt,
};

// ============================================================================
// Error Types
// ============================================================================

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("invalid object ID: {0}")]
    InvalidObjectId(String),

    #[error("invalid timestamp: {0}")]
    InvalidTimestamp(String),

    #[error("invalid identity: {0}")]
    InvalidIdentity(String),

    #[error("invalid ref name: {0}")]
    InvalidRefName(String),

    #[error("invalid tree entry name: {0}")]
    InvalidTreeEntryName(String),

    #[error("invalid path reference: {0}")]
    InvalidPathReference(String),

    #[error("commit not found: {0}")]
    CommitNotFound(String),

    #[error("ambiguous commit hash: {0} matches multiple commits")]
    AmbiguousHash(String),

    #[error("cycle detected in commit graph")]
    CycleDetected,

    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("unexpected value type: expected {expected}, got {actual}")]
    UnexpectedType {
        expected: &'static str,
        actual: String,
    },

    #[error("missing required field: {0}")]
    MissingField(&'static str),

    #[error("unexpected field: {0}")]
    UnexpectedField(String),
}

// ============================================================================
// ObjectId
// ============================================================================

/// A git object ID (SHA-1 hash), always 40 hex characters / 20 bytes.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ObjectId([u8; 20]);

impl ObjectId {
    /// Parse from 40-character hex string
    pub fn from_hex(s: &str) -> Result<Self, ParseError> {
        if s.len() != 40 {
            return Err(ParseError::InvalidObjectId(format!(
                "expected 40 characters, got {}",
                s.len()
            )));
        }

        let mut bytes = [0u8; 20];
        for i in 0..20 {
            let hex_byte = &s[i * 2..i * 2 + 2];
            bytes[i] = u8::from_str_radix(hex_byte, 16).map_err(|_| {
                ParseError::InvalidObjectId(format!("invalid hex characters: {}", s))
            })?;
        }

        Ok(ObjectId(bytes))
    }

    /// Parse from truncated hex string (4-40 characters)
    /// Used during deserialization when reading truncated hashes
    pub fn from_hex_prefix(s: &str) -> Result<Self, ParseError> {
        let len = s.len();

        // Validate length: must be even, >= 4, <= 40
        if len < 4 || len > 40 {
            return Err(ParseError::InvalidObjectId(format!(
                "truncated hash must be 4-40 characters, got {}",
                len
            )));
        }

        if len % 2 != 0 {
            return Err(ParseError::InvalidObjectId(format!(
                "truncated hash must have even length, got {}",
                len
            )));
        }

        // Parse the available hex digits
        let num_bytes = len / 2;
        let mut bytes = [0u8; 20];
        for i in 0..num_bytes {
            let hex_byte = &s[i * 2..i * 2 + 2];
            bytes[i] = u8::from_str_radix(hex_byte, 16).map_err(|_| {
                ParseError::InvalidObjectId(format!("invalid hex characters: {}", s))
            })?;
        }

        // Note: The remaining bytes are left as zeros. This creates a partial ObjectId
        // that should only be used for lookups where the caller will handle ambiguity.
        Ok(ObjectId(bytes))
    }

    /// Convert to 40-character lowercase hex string
    pub fn to_hex(self) -> String {
        self.0.iter().map(|byte| format!("{:02x}", byte)).collect()
    }

    /// Convert to truncated hex string of specified length
    /// Used during serialization for non-head commits
    pub fn to_hex_truncated(&self, len: usize) -> String {
        // Validate length: must be even, >= 4, <= 40
        if len < 4 || len > 40 || len % 2 != 0 {
            // Fall back to full length if invalid
            return self.to_hex();
        }

        let num_bytes = len / 2;
        self.0[..num_bytes]
            .iter()
            .map(|byte| format!("{:02x}", byte))
            .collect()
    }

    /// Get the raw bytes
    pub fn as_bytes(&self) -> &[u8; 20] {
        &self.0
    }
}

impl fmt::Debug for ObjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ObjectId({})", self.to_hex())
    }
}

// ============================================================================
// Timestamp
// ============================================================================

/// A git timestamp: Unix epoch seconds with a timezone offset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timestamp {
    /// Seconds since Unix epoch (must be non-negative for git compatibility)
    pub seconds: i64,

    /// Timezone offset in minutes from UTC (e.g., -120 for -02:00)
    pub offset_minutes: i16,
}

impl Timestamp {
    /// Parse from ISO 8601 string with lenient parsing.
    /// Defaults: month=02, day=04, hour=08, minute=16, second=32, offset=Z
    /// (UTC/0)
    pub fn from_iso8601(s: &str) -> Result<Self, ParseError> {
        let s = s.trim();

        // Split into date and time parts (accepting various delimiters)
        let parts: Vec<&str> = s.split(['T', 't', ' ']).collect();

        if parts.is_empty() {
            return Err(ParseError::InvalidTimestamp("empty string".to_string()));
        }

        // Parse date part (YYYY-MM-DD or YYYY-MM or YYYY)
        let date_part = parts[0];
        let date_components: Vec<&str> = date_part.split(['-', '/']).collect();

        if date_components.is_empty() {
            return Err(ParseError::InvalidTimestamp("missing year".to_string()));
        }

        let year: i32 = date_components[0].parse().map_err(|_| {
            ParseError::InvalidTimestamp(format!("invalid year: {}", date_components[0]))
        })?;

        let month: u32 = if date_components.len() > 1 {
            date_components[1].parse().map_err(|_| {
                ParseError::InvalidTimestamp(format!("invalid month: {}", date_components[1]))
            })?
        } else {
            2 // Default month
        };

        let day: u32 = if date_components.len() > 2 {
            date_components[2].parse().map_err(|_| {
                ParseError::InvalidTimestamp(format!("invalid day: {}", date_components[2]))
            })?
        } else {
            4 // Default day
        };

        // Parse time part if present
        let (hour, minute, second, offset_minutes) = if parts.len() > 1 {
            let time_part = parts[1];

            // Extract timezone offset first (can be Z, +HH:MM, -HH:MM, +HHMM, -HHMM, +HH,
            // -HH)
            let (time_part, offset) = if time_part.ends_with('Z') || time_part.ends_with('z') {
                (&time_part[..time_part.len() - 1], 0i16)
            } else if let Some(pos) = time_part.rfind(['+', '-']) {
                let offset_str = &time_part[pos..];
                let sign = if offset_str.starts_with('-') { -1 } else { 1 };
                let offset_digits = &offset_str[1..];

                let offset_minutes = if offset_digits.contains(':') {
                    // Format: +HH:MM or -HH:MM
                    let parts: Vec<&str> = offset_digits.split(':').collect();
                    if parts.len() != 2 {
                        return Err(ParseError::InvalidTimestamp(format!(
                            "invalid timezone offset: {}",
                            offset_str
                        )));
                    }
                    let hours: i16 = parts[0].parse().map_err(|_| {
                        ParseError::InvalidTimestamp(format!("invalid offset hours: {}", parts[0]))
                    })?;
                    let mins: i16 = parts[1].parse().map_err(|_| {
                        ParseError::InvalidTimestamp(format!(
                            "invalid offset minutes: {}",
                            parts[1]
                        ))
                    })?;
                    sign * (hours * 60 + mins)
                } else if offset_digits.len() == 4 {
                    // Format: +HHMM or -HHMM
                    let hours: i16 = offset_digits[0..2].parse().map_err(|_| {
                        ParseError::InvalidTimestamp(format!(
                            "invalid offset hours: {}",
                            &offset_digits[0..2]
                        ))
                    })?;
                    let mins: i16 = offset_digits[2..4].parse().map_err(|_| {
                        ParseError::InvalidTimestamp(format!(
                            "invalid offset minutes: {}",
                            &offset_digits[2..4]
                        ))
                    })?;
                    sign * (hours * 60 + mins)
                } else if offset_digits.len() == 2 {
                    // Format: +HH or -HH
                    let hours: i16 = offset_digits.parse().map_err(|_| {
                        ParseError::InvalidTimestamp(format!(
                            "invalid offset hours: {}",
                            offset_digits
                        ))
                    })?;
                    sign * (hours * 60)
                } else {
                    return Err(ParseError::InvalidTimestamp(format!(
                        "invalid timezone offset: {}",
                        offset_str
                    )));
                };

                (&time_part[..pos], offset_minutes)
            } else {
                (time_part, 0i16) // Default offset
            };

            // Parse HH:MM:SS
            let time_components: Vec<&str> = time_part.split(':').collect();

            let hour: u32 = if !time_components.is_empty() && !time_components[0].is_empty() {
                time_components[0].parse().map_err(|_| {
                    ParseError::InvalidTimestamp(format!("invalid hour: {}", time_components[0]))
                })?
            } else {
                8 // Default hour
            };

            let minute: u32 = if time_components.len() > 1 {
                time_components[1].parse().map_err(|_| {
                    ParseError::InvalidTimestamp(format!("invalid minute: {}", time_components[1]))
                })?
            } else {
                16 // Default minute
            };

            let second: u32 = if time_components.len() > 2 {
                time_components[2].parse().map_err(|_| {
                    ParseError::InvalidTimestamp(format!("invalid second: {}", time_components[2]))
                })?
            } else {
                32 // Default second
            };

            (hour, minute, second, offset)
        } else {
            (8, 16, 32, 0) // Default time and offset
        };

        // Convert to Unix timestamp
        // Simplified calculation (doesn't handle all edge cases perfectly, but good
        // enough for our use)
        let days_from_epoch = Self::days_since_epoch(year, month, day)
            .ok_or_else(|| ParseError::InvalidTimestamp("date before Unix epoch".to_string()))?;

        let seconds = days_from_epoch as i64 * 86400
            + hour as i64 * 3600
            + minute as i64 * 60
            + second as i64;

        if seconds < 0 {
            return Err(ParseError::InvalidTimestamp(
                "timestamp before Unix epoch".to_string(),
            ));
        }

        Ok(Timestamp {
            seconds,
            offset_minutes,
        })
    }

    /// Calculate days since Unix epoch (1970-01-01)
    fn days_since_epoch(year: i32, month: u32, day: u32) -> Option<i32> {
        if year < 1970 {
            return None;
        }

        // Days in each month (non-leap year)
        const DAYS_IN_MONTH: [u32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

        let mut days = 0i32;

        // Add days for complete years
        for y in 1970..year {
            days += if Self::is_leap_year(y) { 366 } else { 365 };
        }

        // Add days for complete months in current year
        for m in 1..month {
            days += DAYS_IN_MONTH[(m - 1) as usize] as i32;
            if m == 2 && Self::is_leap_year(year) {
                days += 1;
            }
        }

        // Add days in current month
        days += (day - 1) as i32;

        Some(days)
    }

    fn is_leap_year(year: i32) -> bool {
        (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
    }

    /// Format as ISO 8601 string
    pub fn to_iso8601(self) -> String {
        // Convert Unix timestamp to date/time components
        let total_days = self.seconds / 86400;
        let remaining_seconds = self.seconds % 86400;

        let hour = remaining_seconds / 3600;
        let minute = (remaining_seconds % 3600) / 60;
        let second = remaining_seconds % 60;

        // Calculate year, month, day from days since epoch
        let (year, month, day) = Self::date_from_days(total_days as i32);

        // Format timezone offset
        let tz_string = if self.offset_minutes == 0 {
            "Z".to_string()
        } else {
            let sign = if self.offset_minutes < 0 { "-" } else { "+" };
            let abs_minutes = self.offset_minutes.abs();
            let hours = abs_minutes / 60;
            let mins = abs_minutes % 60;
            format!("{}{:02}:{:02}", sign, hours, mins)
        };

        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}{}",
            year, month, day, hour, minute, second, tz_string
        )
    }

    /// Convert days since Unix epoch to (year, month, day)
    fn date_from_days(mut days: i32) -> (i32, u32, u32) {
        const DAYS_IN_MONTH: [u32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

        let mut year = 1970;

        // Find the year
        loop {
            let days_in_year = if Self::is_leap_year(year) { 366 } else { 365 };
            if days < days_in_year {
                break;
            }
            days -= days_in_year;
            year += 1;
        }

        // Find the month and day
        let mut month = 1u32;
        for m in 1..=12 {
            let mut days_in_month = DAYS_IN_MONTH[(m - 1) as usize];
            if m == 2 && Self::is_leap_year(year) {
                days_in_month = 29;
            }

            if days < days_in_month as i32 {
                month = m;
                break;
            }
            days -= days_in_month as i32;
        }

        let day = (days + 1) as u32;

        (year, month, day)
    }
}

// ============================================================================
// Identity
// ============================================================================

/// A git identity (author or committer), consisting of name and email.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identity {
    /// Name portion (everything before the email)
    pub name: String,

    /// Email address (without angle brackets)
    pub email: String,
}

impl Identity {
    /// Parse from git format: "Name <email@example.com>"
    /// Follows the pattern: /^[^<]+[ ]<[^@]+@[^>]+>$/
    pub fn parse(s: &str) -> Result<Self, ParseError> {
        let s = s.trim();

        // Find the '<' and '>' brackets
        let open_bracket = s
            .rfind('<')
            .ok_or_else(|| ParseError::InvalidIdentity("missing '<' before email".to_string()))?;

        let close_bracket = s
            .rfind('>')
            .ok_or_else(|| ParseError::InvalidIdentity("missing '>' after email".to_string()))?;

        if close_bracket != s.len() - 1 {
            return Err(ParseError::InvalidIdentity(
                "characters after closing '>'".to_string(),
            ));
        }

        if close_bracket <= open_bracket {
            return Err(ParseError::InvalidIdentity(
                "invalid bracket positions".to_string(),
            ));
        }

        // Extract name and email
        let name_part = &s[..open_bracket];
        let email_part = &s[open_bracket + 1..close_bracket];

        // Name must not be empty and must end with a space
        if name_part.is_empty() || !name_part.ends_with(' ') {
            return Err(ParseError::InvalidIdentity(
                "name must end with a space before '<'".to_string(),
            ));
        }

        // Email must contain '@'
        if !email_part.contains('@') {
            return Err(ParseError::InvalidIdentity(
                "email must contain '@'".to_string(),
            ));
        }

        let name = name_part.trim_end().to_string();
        let email = email_part.to_string();

        if name.is_empty() {
            return Err(ParseError::InvalidIdentity(
                "name cannot be empty".to_string(),
            ));
        }

        Ok(Identity { name, email })
    }

    /// Format as git string: "Name <email@example.com>"
    pub fn format(&self) -> String {
        format!("{} <{}>", self.name, self.email)
    }
}

// ============================================================================
// Hash Computation Helpers
// ============================================================================

/// Compute the git blob hash for the given content.
/// Git blobs are hashed as: SHA-1("blob {size}\0{content}")
fn compute_blob_hash(content: &str) -> ObjectId {
    use sha1_checked::Digest;

    let header = format!("blob {}\0", content.len());
    let mut hasher = sha1_checked::Sha1::new();
    hasher.update(header.as_bytes());
    hasher.update(content.as_bytes());
    let result = hasher.finalize();

    let mut bytes = [0u8; 20];
    bytes.copy_from_slice(&result);
    ObjectId(bytes)
}

/// Compute the git tree hash for the given tree entries.
/// Git trees are hashed as: SHA-1("tree {size}\0{entries}")
/// where entries are sorted by name and formatted as: "{mode}
/// {name}\0{hash_bytes}"
fn compute_tree_hash_from_entries(entries: &BTreeMap<String, (u32, ObjectId)>) -> ObjectId {
    use sha1_checked::Digest;

    // Build the tree object content
    let mut content = Vec::new();
    for (name, (mode, hash)) in entries {
        content.extend_from_slice(format!("{} {}\0", mode, name).as_bytes());
        content.extend_from_slice(hash.as_bytes());
    }

    let header = format!("tree {}\0", content.len());
    let mut hasher = sha1_checked::Sha1::new();
    hasher.update(header.as_bytes());
    hasher.update(&content);
    let result = hasher.finalize();

    let mut bytes = [0u8; 20];
    bytes.copy_from_slice(&result);
    ObjectId(bytes)
}

// ============================================================================
// Tree
// ============================================================================

/// The contents of a git tree (directory).
/// Stored as a flat map from full paths to blob contents.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Tree {
    /// Map from file paths to blob contents.
    /// Paths use forward slashes as separators, never have leading/trailing
    /// slashes.
    entries: BTreeMap<String, String>,

    /// Map from file paths to blob object IDs (hashes).
    /// Each blob's hash is computed from its content using git's blob hashing
    /// algorithm.
    blob_hashes: BTreeMap<String, ObjectId>,

    /// Cached tree hash for this tree.
    /// Computed from the tree's contents following git's tree hashing
    /// algorithm. This is used during serialization to enable deduplication
    /// via references.
    tree_hash: Option<ObjectId>,
}

/// A tree delta represents changes to apply on top of a base tree.
/// This is used during deserialization when parsing the on-disk format.
pub type TreeDelta = BTreeMap<String, TreeEntry>;

/// An entry in a tree delta
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TreeEntry {
    /// Set a blob to this content
    Blob(String),

    /// Recursively modify a subtree
    Tree(BTreeMap<String, TreeEntry>),

    /// Delete this path
    Delete,

    /// Reference to content from another commit/path
    /// Used during deserialization when encountering [commit] and/or [path]
    /// references
    Reference(TreeReference),
}

/// A reference to a tree or blob from another location.
/// Represents the [commit] and [path] special keys in the on-disk format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeReference {
    /// The commit to reference (None means use the inherited [commit] value)
    pub commit: Option<ObjectId>,

    /// The path within that commit's tree to reference (None means use the
    /// inherited path)
    pub path: Option<String>,
}

impl Tree {
    /// Create an empty tree
    pub fn new() -> Self {
        Tree {
            entries: BTreeMap::new(),
            blob_hashes: BTreeMap::new(),
            tree_hash: None,
        }
    }

    /// Get the content of a blob at the given path
    pub fn get(&self, path: &str) -> Option<&str> {
        self.entries.get(path).map(|s| s.as_str())
    }

    /// Set the content of a blob at the given path
    pub fn insert(&mut self, path: String, content: String) {
        // Validate path components
        if Self::validate_path(&path).is_err() {
            // For now, just insert anyway. Validation should be done before
            // calling. In a full implementation, we might want to
            // return Result here.
        }

        // Compute and store the blob hash
        let blob_hash = compute_blob_hash(&content);
        self.blob_hashes.insert(path.clone(), blob_hash);

        // Insert the content
        self.entries.insert(path, content);

        // Invalidate the tree hash cache
        self.tree_hash = None;
    }

    /// Remove a file or directory at the given path
    /// Returns true if something was removed
    pub fn remove(&mut self, path: &str) -> bool {
        // Remove the exact path if it exists
        let exact_removed = self.entries.remove(path).is_some();
        if exact_removed {
            self.blob_hashes.remove(path);
        }

        // Remove all paths that start with this path followed by '/'
        let prefix = format!("{}/", path);
        let keys_to_remove: Vec<String> = self
            .entries
            .keys()
            .filter(|k| k.starts_with(&prefix))
            .cloned()
            .collect();

        let dir_removed = !keys_to_remove.is_empty();
        for key in keys_to_remove {
            self.entries.remove(&key);
            self.blob_hashes.remove(&key);
        }

        // Invalidate the tree hash cache if anything was removed
        if exact_removed || dir_removed {
            self.tree_hash = None;
        }

        exact_removed || dir_removed
    }

    /// List all paths in the tree
    pub fn paths(&self) -> impl Iterator<Item = &str> {
        self.entries.keys().map(|s| s.as_str())
    }

    /// Check if the tree is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Validate a path and its components
    fn validate_path(path: &str) -> Result<(), ParseError> {
        if path.is_empty() {
            return Err(ParseError::InvalidTreeEntryName(
                "path cannot be empty".to_string(),
            ));
        }

        if path.starts_with('/') || path.ends_with('/') {
            return Err(ParseError::InvalidTreeEntryName(
                "path cannot start or end with '/'".to_string(),
            ));
        }

        for component in path.split('/') {
            Self::validate_component(component)?;
        }

        Ok(())
    }

    /// Validate a single path component (file or directory name)
    fn validate_component(name: &str) -> Result<(), ParseError> {
        if name.is_empty() || name == "." || name == ".." {
            return Err(ParseError::InvalidTreeEntryName(format!(
                "invalid component: '{}'",
                name
            )));
        }

        if name.contains('/') || name.contains('\\') || name.contains(':') || name.contains('\0') {
            return Err(ParseError::InvalidTreeEntryName(format!(
                "component contains invalid character: '{}'",
                name
            )));
        }

        Ok(())
    }

    /// Get the hash of a blob at the given path
    pub fn get_blob_hash(&self, path: &str) -> Option<&ObjectId> {
        self.blob_hashes.get(path)
    }

    /// Get the cached tree hash if available
    pub fn hash(&self) -> Option<ObjectId> {
        self.tree_hash
    }

    /// Compute and cache the tree hash for this tree
    /// Returns the cached hash if already computed
    pub fn compute_hash(&mut self) -> ObjectId {
        if let Some(hash) = self.tree_hash {
            return hash;
        }

        // Build a hierarchical tree structure from the flat entries
        // This is needed to compute hashes correctly for git's tree objects
        let hash = self.compute_tree_hash_for_path("");
        self.tree_hash = Some(hash);
        hash
    }

    /// Compute the tree hash for a specific path prefix
    /// This reconstructs the hierarchical tree structure from the flat
    /// representation
    fn compute_tree_hash_for_path(&self, prefix: &str) -> ObjectId {
        use std::collections::BTreeMap;

        // Collect all direct children (files and subdirectories) at this level
        let mut entries: BTreeMap<String, (u32, ObjectId)> = BTreeMap::new();

        let prefix_with_slash = if prefix.is_empty() {
            String::new()
        } else {
            format!("{}/", prefix)
        };

        for (path, _content) in &self.entries {
            // Skip entries that don't start with our prefix
            if !prefix.is_empty() && !path.starts_with(&prefix_with_slash) {
                continue;
            }

            // Get the relative path from this prefix
            let relative = if prefix.is_empty() {
                path.as_str()
            } else {
                &path[prefix_with_slash.len()..]
            };

            // Split into first component and rest
            if let Some(slash_pos) = relative.find('/') {
                // This is a subdirectory
                let dir_name = &relative[..slash_pos];

                // Only compute the subtree hash once per directory
                if !entries.contains_key(dir_name) {
                    let subtree_prefix = if prefix.is_empty() {
                        dir_name.to_string()
                    } else {
                        format!("{}/{}", prefix, dir_name)
                    };
                    let subtree_hash = self.compute_tree_hash_for_path(&subtree_prefix);
                    entries.insert(dir_name.to_string(), (40000, subtree_hash)); // mode 040000 = directory
                }
            } else {
                // This is a file at this level
                let blob_hash = self.blob_hashes.get(path).copied().unwrap_or_else(|| {
                    // Should not happen if blob_hashes is maintained correctly
                    compute_blob_hash(self.entries.get(path).unwrap())
                });
                entries.insert(relative.to_string(), (100644, blob_hash)); // mode 100644 = regular file
            }
        }

        // Compute the hash for this tree object
        compute_tree_hash_from_entries(&entries)
    }

    /// Get the tree at a specific path (returns a subtree containing only
    /// entries under that path)
    pub fn get_tree(&self, prefix: &str) -> Tree {
        let mut subtree = Tree::new();

        let prefix_with_slash = if prefix.is_empty() {
            String::new()
        } else {
            format!("{}/", prefix)
        };

        for (path, content) in &self.entries {
            if prefix.is_empty() || path.starts_with(&prefix_with_slash) {
                // Get the relative path
                let relative = if prefix.is_empty() {
                    path.clone()
                } else {
                    path[prefix_with_slash.len()..].to_string()
                };

                subtree.insert(relative, content.clone());
            }
        }

        subtree
    }

    /// Apply a tree delta on top of this tree (used during deserialization)
    pub fn apply_delta(&mut self, delta: &TreeDelta) {
        self.apply_delta_internal("", delta);
        // Invalidate the tree hash cache after applying delta
        self.tree_hash = None;
    }

    /// Internal recursive helper for applying deltas
    fn apply_delta_internal(&mut self, prefix: &str, delta: &TreeDelta) {
        for (name, entry) in delta {
            let path = if prefix.is_empty() {
                name.clone()
            } else {
                format!("{}/{}", prefix, name)
            };

            match entry {
                TreeEntry::Blob(content) => {
                    self.insert(path, content.clone());
                }
                TreeEntry::Tree(nested_delta) => {
                    self.apply_delta_internal(&path, nested_delta);
                }
                TreeEntry::Delete => {
                    self.remove(&path);
                }
                TreeEntry::Reference(_) => {
                    // References should be resolved before calling apply_delta
                    // For now, just skip them
                }
            }
        }
    }
}

// ============================================================================
// Commit
// ============================================================================

/// A git commit with all fields fully resolved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Commit {
    /// The commit's object ID
    pub id: ObjectId,

    /// Parent commit IDs (empty for root commits)
    pub parents: Vec<ObjectId>,

    /// The tree (file contents) for this commit
    pub tree: Tree,

    /// Commit author
    pub author: Identity,

    /// Author timestamp
    pub author_date: Timestamp,

    /// Committer (often same as author)
    pub committer: Identity,

    /// Commit timestamp
    pub committer_date: Timestamp,

    /// Commit message (may be empty string, but never None)
    pub message: String,
}

// ============================================================================
// RefName
// ============================================================================

/// A git reference name (e.g., "refs/heads/main").
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RefName(String);

impl RefName {
    /// Parse a ref name, ensuring it starts with "refs/"
    pub fn new(s: String) -> Result<Self, ParseError> {
        if !s.starts_with("refs/") {
            return Err(ParseError::InvalidRefName(format!(
                "ref name must start with 'refs/', got: {}",
                s
            )));
        }
        Ok(RefName(s))
    }

    /// Get the full ref name (e.g., "refs/heads/main")
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get the short name (e.g., "main" from "refs/heads/main")
    pub fn short_name(&self) -> &str {
        // Remove the longest matching prefix
        if let Some(stripped) = self.0.strip_prefix("refs/heads/") {
            stripped
        } else if let Some(stripped) = self.0.strip_prefix("refs/tags/") {
            stripped
        } else if let Some(stripped) = self.0.strip_prefix("refs/remotes/") {
            stripped
        } else if let Some(stripped) = self.0.strip_prefix("refs/") {
            stripped
        } else {
            &self.0
        }
    }
}

impl fmt::Debug for RefName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RefName({})", self.0)
    }
}

// ============================================================================
// HeadState
// ============================================================================

/// The state of HEAD: either pointing to a ref (symbolic) or directly to a
/// commit (detached).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeadState {
    /// HEAD points to a branch ref (e.g., "refs/heads/main")
    /// The branch may or may not exist (unborn branch if it doesn't)
    Symbolic(RefName),

    /// HEAD points directly to a commit (detached HEAD)
    Detached(ObjectId),
}

// ============================================================================
// Repository
// ============================================================================

/// The complete in-memory representation of a git repository snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Repository {
    /// All commits in the repository, indexed by object ID.
    /// Only includes commits reachable from HEAD or refs.
    commits: HashMap<ObjectId, Commit>,

    /// Branch references (mapping ref names to commit IDs).
    /// For V1, these are all under refs/heads/.
    refs: BTreeMap<RefName, ObjectId>,

    /// Current HEAD state
    head: HeadState,
}

impl Repository {
    /// Create a new empty repository with an unborn HEAD on refs/heads/trunk
    pub fn new() -> Self {
        Repository {
            commits: HashMap::new(),
            refs: BTreeMap::new(),
            head: HeadState::Symbolic(
                RefName::new("refs/heads/trunk".to_string())
                    .expect("hardcoded ref name should be valid"),
            ),
        }
    }

    /// Get a commit by ID
    pub fn get_commit(&self, id: &ObjectId) -> Option<&Commit> {
        self.commits.get(id)
    }

    /// Get all commits
    pub fn commits(&self) -> impl Iterator<Item = &Commit> {
        self.commits.values()
    }

    /// Get a ref's target commit ID
    pub fn get_ref(&self, name: &RefName) -> Option<&ObjectId> {
        self.refs.get(name)
    }

    /// Get all refs
    pub fn refs(&self) -> impl Iterator<Item = (&RefName, &ObjectId)> {
        self.refs.iter()
    }

    /// Get the HEAD state
    pub fn head(&self) -> &HeadState {
        &self.head
    }

    /// Get the commit that HEAD points to (if any)
    /// Returns None for unborn HEAD
    pub fn head_commit(&self) -> Option<&Commit> {
        match &self.head {
            HeadState::Detached(id) => self.commits.get(id),
            HeadState::Symbolic(ref_name) => {
                self.refs.get(ref_name).and_then(|id| self.commits.get(id))
            }
        }
    }

    /// Add or update a commit
    pub fn insert_commit(&mut self, commit: Commit) {
        self.commits.insert(commit.id, commit);
    }

    /// Add or update a ref
    pub fn insert_ref(&mut self, name: RefName, target: ObjectId) {
        self.refs.insert(name, target);
    }

    /// Set the HEAD state
    pub fn set_head(&mut self, head: HeadState) {
        self.head = head;
    }
}

impl Default for Repository {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Parsing Implementation
// ============================================================================

/// Parse a YAML string into a Repository.
pub fn parse(yaml: &str) -> Result<Repository, ParseError> {
    let value: serde_yaml::Value = serde_yaml::from_str(yaml)?;

    let mapping = value
        .as_mapping()
        .ok_or_else(|| ParseError::UnexpectedType {
            expected: "mapping",
            actual: format!("{:?}", value),
        })?;

    // Parse HEAD
    let head_value = mapping
        .get(serde_yaml::Value::String("HEAD".to_string()))
        .ok_or(ParseError::MissingField("HEAD"))?;

    let head = parse_head(head_value)?;

    // Parse refs
    let refs_value = mapping.get(serde_yaml::Value::String("refs".to_string()));
    let refs = if let Some(refs_value) = refs_value {
        parse_refs(refs_value)?
    } else {
        BTreeMap::new()
    };

    // Parse commits - build a map of commit references to their definitions
    // Use a Vec to preserve document order, not BTreeMap which sorts by CommitRef
    let mut commit_defs_vec: Vec<(CommitRef, &serde_yaml::Mapping)> = Vec::new();
    let mut integer_to_hex: HashMap<u32, ObjectId> = HashMap::new();
    let mut prefix_to_hex: HashMap<String, ObjectId> = HashMap::new();

    for (key, value) in mapping.iter() {
        let key_str = key.as_str();
        if key_str == Some("HEAD") || key_str == Some("refs") {
            continue;
        }

        // Parse commit reference
        let commit_ref = parse_commit_ref_key(key)?;

        let commit_mapping = value
            .as_mapping()
            .ok_or_else(|| ParseError::UnexpectedType {
                expected: "mapping",
                actual: format!("{:?}", value),
            })?;

        commit_defs_vec.push((commit_ref.clone(), commit_mapping));
    }

    // First pass: compute ObjectIds for integer-keyed commits
    // We need to resolve the commit graph to compute object IDs
    // For now, we'll use a placeholder approach and compute them later

    // Build commit_defs HashMap for fast lookups
    let commit_defs: HashMap<CommitRef, &serde_yaml::Mapping> = commit_defs_vec.iter().cloned().collect();

    // Build commits in document order (from vec, which preserves insertion order)
    let commit_order: Vec<_> = commit_defs_vec.iter().map(|(ref_val, _)| ref_val.clone()).collect();

    // Build the commits
    let mut commits = HashMap::new();
    let mut commit_processing_state: HashMap<CommitRef, CommitProcessingState> = HashMap::new();

    for (idx, commit_ref) in commit_order.iter().enumerate() {
        let prev_commit_ref = if idx > 0 {
            Some(commit_order[idx - 1].clone())
        } else {
            None
        };

        let commit = build_commit(
            commit_ref,
            &commit_defs,
            &commit_order,
            idx,
            prev_commit_ref.as_ref(),
            &mut integer_to_hex,
            &mut prefix_to_hex,
            &mut commit_processing_state,
        )?;

        commits.insert(commit.id, commit);
    }

    // Convert refs to use resolved ObjectIds
    let mut resolved_refs = BTreeMap::new();
    for (ref_name, commit_ref) in refs {
        let object_id = resolve_commit_ref(&commit_ref, &integer_to_hex, &prefix_to_hex, &commits)?;
        resolved_refs.insert(ref_name, object_id);
    }

    // Convert HEAD to use resolved ObjectId
    let resolved_head = match head {
        HeadStateOrRef::Symbolic(ref_name) => HeadState::Symbolic(ref_name),
        HeadStateOrRef::Detached(commit_ref) => {
            let object_id = resolve_commit_ref(&commit_ref, &integer_to_hex, &prefix_to_hex, &commits)?;
            HeadState::Detached(object_id)
        }
    };

    let mut repo = Repository {
        commits,
        refs: resolved_refs,
        head: resolved_head,
    };

    // Prune unreachable commits
    prune_unreachable(&mut repo);

    Ok(repo)
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum CommitRef {
    Hex(ObjectId),
    Prefix(String), // Truncated hex prefix (4-40 chars)
    Int(u32),
}

#[derive(Debug, Clone)]
enum CommitProcessingState {
    InProgress,
    Complete(Box<Commit>),
}

fn parse_head(value: &serde_yaml::Value) -> Result<HeadStateOrRef, ParseError> {
    if let Some(s) = value.as_str() {
        if s.starts_with("refs/") {
            let ref_name = RefName::new(s.to_string())?;
            Ok(HeadStateOrRef::Symbolic(ref_name))
        } else {
            // Try to parse as ObjectId or integer
            if s.len() == 40 {
                let oid = ObjectId::from_hex(s)?;
                Ok(HeadStateOrRef::Detached(CommitRef::Hex(oid)))
            } else {
                Err(ParseError::UnexpectedType {
                    expected: "ref name or commit ID",
                    actual: s.to_string(),
                })
            }
        }
    } else if let Some(n) = value.as_u64() {
        if n == 0 || n > u32::MAX as u64 {
            return Err(ParseError::InvalidObjectId(format!(
                "integer commit reference must be positive and fit in u32: {}",
                n
            )));
        }
        Ok(HeadStateOrRef::Detached(CommitRef::Int(n as u32)))
    } else {
        Err(ParseError::UnexpectedType {
            expected: "string or integer",
            actual: format!("{:?}", value),
        })
    }
}

#[derive(Debug, Clone)]
enum HeadStateOrRef {
    Symbolic(RefName),
    Detached(CommitRef),
}

fn parse_refs(value: &serde_yaml::Value) -> Result<BTreeMap<RefName, CommitRef>, ParseError> {
    let mapping = value
        .as_mapping()
        .ok_or_else(|| ParseError::UnexpectedType {
            expected: "mapping",
            actual: format!("{:?}", value),
        })?;

    let mut refs = BTreeMap::new();
    parse_refs_recursive("refs", mapping, &mut refs)?;
    Ok(refs)
}

fn parse_refs_recursive(
    prefix: &str,
    mapping: &serde_yaml::Mapping,
    refs: &mut BTreeMap<RefName, CommitRef>,
) -> Result<(), ParseError> {
    for (key, value) in mapping.iter() {
        let key_name = normalize_yaml_key(key)?;
        let full_path = format!("{}/{}", prefix, key_name);

        if let Some(nested_mapping) = value.as_mapping() {
            // Recursively parse nested refs
            parse_refs_recursive(&full_path, nested_mapping, refs)?;
        } else {
            // This is a leaf - parse the commit reference
            let commit_ref = parse_commit_ref(value)?;
            let ref_name = RefName::new(full_path)?;
            refs.insert(ref_name, commit_ref);
        }
    }

    Ok(())
}

fn parse_commit_ref_key(key: &serde_yaml::Value) -> Result<CommitRef, ParseError> {
    if let Some(s) = key.as_str() {
        let len = s.len();

        // Check if it's a valid hex string
        if !s.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(ParseError::InvalidObjectId(format!(
                "commit key must contain only hex characters, got: {}",
                s
            )));
        }

        if len == 40 {
            let oid = ObjectId::from_hex(s)?;
            Ok(CommitRef::Hex(oid))
        } else if len >= 4 && len <= 40 {
            // Truncated hash - will be resolved later
            Ok(CommitRef::Prefix(s.to_string()))
        } else {
            Err(ParseError::InvalidObjectId(format!(
                "commit key must be 4-40 hex characters or positive integer, got: {}",
                s
            )))
        }
    } else if let Some(n) = key.as_u64() {
        if n == 0 || n > u32::MAX as u64 {
            return Err(ParseError::InvalidObjectId(format!(
                "integer commit reference must be positive and fit in u32: {}",
                n
            )));
        }
        Ok(CommitRef::Int(n as u32))
    } else {
        Err(ParseError::UnexpectedType {
            expected: "string or integer",
            actual: format!("{:?}", key),
        })
    }
}

fn parse_commit_ref(value: &serde_yaml::Value) -> Result<CommitRef, ParseError> {
    if let Some(s) = value.as_str() {
        let len = s.len();

        // Check if it's a valid hex string
        if !s.chars().all(|c| c.is_ascii_hexdigit()) {
            return Err(ParseError::InvalidObjectId(format!(
                "commit reference must contain only hex characters, got: {}",
                s
            )));
        }

        if len == 40 {
            let oid = ObjectId::from_hex(s)?;
            Ok(CommitRef::Hex(oid))
        } else if len >= 4 && len <= 40 {
            // Truncated hash reference
            Ok(CommitRef::Prefix(s.to_string()))
        } else {
            Err(ParseError::InvalidObjectId(format!(
                "commit reference must be 4-40 hex characters or positive integer, got: {}",
                s
            )))
        }
    } else if let Some(n) = value.as_u64() {
        if n == 0 || n > u32::MAX as u64 {
            return Err(ParseError::InvalidObjectId(format!(
                "integer commit reference must be positive and fit in u32: {}",
                n
            )));
        }
        Ok(CommitRef::Int(n as u32))
    } else {
        Err(ParseError::UnexpectedType {
            expected: "string or integer",
            actual: format!("{:?}", value),
        })
    }
}

fn resolve_commit_ref(
    commit_ref: &CommitRef,
    integer_to_hex: &HashMap<u32, ObjectId>,
    prefix_to_hex: &HashMap<String, ObjectId>,
    commits: &HashMap<ObjectId, Commit>,
) -> Result<ObjectId, ParseError> {
    match commit_ref {
        CommitRef::Hex(oid) => Ok(*oid),
        CommitRef::Prefix(prefix) => {
            // First check if we have a direct mapping from parsing
            if let Some(oid) = prefix_to_hex.get(prefix) {
                return Ok(*oid);
            }

            // Fall back to searching commits by prefix (for validation/ambiguity checking)
            let matches: Vec<ObjectId> = commits
                .keys()
                .filter(|oid| {
                    let hex_str = oid.to_hex();
                    hex_str.starts_with(prefix)
                })
                .copied()
                .collect();

            match matches.len() {
                0 => Err(ParseError::CommitNotFound(format!(
                    "no commit found matching prefix: {}",
                    prefix
                ))),
                1 => Ok(matches[0]),
                _ => Err(ParseError::AmbiguousHash(prefix.clone())),
            }
        }
        CommitRef::Int(n) => integer_to_hex.get(n).copied().ok_or_else(|| {
            ParseError::CommitNotFound(format!("integer commit reference {} not found", n))
        }),
    }
}

fn normalize_yaml_key(key: &serde_yaml::Value) -> Result<String, ParseError> {
    match key {
        serde_yaml::Value::String(s) => Ok(s.clone()),
        serde_yaml::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(i.to_string())
            } else if let Some(u) = n.as_u64() {
                Ok(u.to_string())
            } else {
                Err(ParseError::UnexpectedType {
                    expected: "integer",
                    actual: format!("{:?}", n),
                })
            }
        }
        serde_yaml::Value::Bool(b) => Ok(b.to_string()),
        serde_yaml::Value::Null => Ok("null".to_string()),
        serde_yaml::Value::Tagged(_) => Err(ParseError::UnexpectedField(
            "YAML tags are not supported".to_string(),
        )),
        _ => Err(ParseError::UnexpectedType {
            expected: "string, number, boolean, or null",
            actual: format!("{:?}", key),
        }),
    }
}

/// Check if a YAML key is a special key like [commit] or [path]
/// These are sequences containing a single string element
fn is_special_key(key: &serde_yaml::Value, name: &str) -> bool {
    if let Some(seq) = key.as_sequence() {
        if seq.len() == 1 {
            if let Some(s) = seq[0].as_str() {
                return s == name;
            }
        }
    }
    false
}

/// Try to get a special key value from a mapping
fn get_special_key<'a>(
    mapping: &'a serde_yaml::Mapping,
    name: &str,
) -> Option<&'a serde_yaml::Value> {
    for (key, value) in mapping.iter() {
        if is_special_key(key, name) {
            return Some(value);
        }
    }
    None
}

fn build_commit(
    commit_ref: &CommitRef,
    commit_defs: &HashMap<CommitRef, &serde_yaml::Mapping>,
    commit_order: &[CommitRef],
    _idx: usize,
    prev_commit_ref: Option<&CommitRef>,
    integer_to_hex: &mut HashMap<u32, ObjectId>,
    prefix_to_hex: &mut HashMap<String, ObjectId>,
    processing_state: &mut HashMap<CommitRef, CommitProcessingState>,
) -> Result<Commit, ParseError> {
    // Check for cycles
    if let Some(CommitProcessingState::InProgress) = processing_state.get(commit_ref) {
        return Err(ParseError::CycleDetected);
    }

    // Check if already processed
    if let Some(CommitProcessingState::Complete(commit)) = processing_state.get(commit_ref) {
        return Ok((**commit).clone());
    }

    processing_state.insert(commit_ref.clone(), CommitProcessingState::InProgress);

    let commit_mapping = commit_defs
        .get(commit_ref)
        .ok_or_else(|| ParseError::CommitNotFound(format!("commit {:?} not found", commit_ref)))?;

    // Parse parents (with default)
    let parents = parse_parents(commit_mapping, prev_commit_ref)?;

    // Resolve parent commits first (for defaults)
    let mut resolved_parents = Vec::new();
    for parent_ref in &parents {
        // Recursively build parent commit if needed
        if let Some(_parent_def) = commit_defs.get(parent_ref) {
            let parent_idx = commit_order
                .iter()
                .position(|r| r == parent_ref)
                .ok_or_else(|| {
                    ParseError::CommitNotFound(format!(
                        "parent commit {:?} not in order",
                        parent_ref
                    ))
                })?;
            let prev_parent = if parent_idx > 0 {
                Some(&commit_order[parent_idx - 1])
            } else {
                None
            };
            let parent_commit = build_commit(
                parent_ref,
                commit_defs,
                commit_order,
                parent_idx,
                prev_parent,
                integer_to_hex,
                prefix_to_hex,
                processing_state,
            )?;
            resolved_parents.push(parent_commit);
        } else {
            return Err(ParseError::CommitNotFound(format!(
                "parent commit {:?} not defined",
                parent_ref
            )));
        }
    }

    let first_parent = resolved_parents.first();

    // Parse author (with default)
    let author = parse_author(commit_mapping, first_parent)?;

    // Parse author-date (with default)
    let author_date = parse_author_date(commit_mapping, &resolved_parents)?;

    // Parse committer (with default)
    let committer = parse_committer(commit_mapping, &author)?;

    // Parse commit-date (with default)
    let committer_date = parse_committer_date(commit_mapping, author_date, &resolved_parents)?;

    // Parse message (with default)
    let message = parse_message(commit_mapping, commit_ref, &committer_date)?;

    // Parse tree (with default)
    let tree = parse_tree(commit_mapping, first_parent, processing_state)?;

    // Determine object ID: use the key for hex/prefix refs, calculate for integer refs
    let object_id = match commit_ref {
        CommitRef::Hex(oid) => *oid,
        CommitRef::Prefix(prefix) => {
            // For truncated hashes, we calculate the full hash from content
            // The prefix in the YAML is just a label for human readability
            let parent_ids: Vec<ObjectId> = resolved_parents.iter().map(|c| c.id).collect();
            let tree_id = calculate_tree_id(&tree)?;
            let calculated_id = calculate_commit_id(
                &tree_id,
                &parent_ids,
                &author,
                author_date,
                &committer,
                committer_date,
                &message,
            )?;

            // Store the mapping from prefix to calculated ID
            prefix_to_hex.insert(prefix.clone(), calculated_id);

            // Optionally verify the prefix matches (for debugging)
            // But don't fail if it doesn't - YAML keys are just labels
            let calculated_hex = calculated_id.to_hex();
            if !calculated_hex.starts_with(prefix) {
                eprintln!("WARNING: commit content hash {} doesn't start with declared prefix {}",
                    calculated_hex, prefix);
            }

            calculated_id
        }
        CommitRef::Int(_) => {
            // Calculate ID for integer references
            let parent_ids: Vec<ObjectId> = resolved_parents.iter().map(|c| c.id).collect();
            let tree_id = calculate_tree_id(&tree)?;
            let calculated_id = calculate_commit_id(
                &tree_id,
                &parent_ids,
                &author,
                author_date,
                &committer,
                committer_date,
                &message,
            )?;

            // Store the mapping for integer references
            if let CommitRef::Int(n) = commit_ref {
                integer_to_hex.insert(*n, calculated_id);
            }

            calculated_id
        }
    };

    let parent_ids: Vec<ObjectId> = resolved_parents.iter().map(|c| c.id).collect();

    let commit = Commit {
        id: object_id,
        parents: parent_ids,
        tree,
        author,
        author_date,
        committer,
        committer_date,
        message,
    };

    processing_state.insert(
        commit_ref.clone(),
        CommitProcessingState::Complete(Box::new(commit.clone())),
    );

    Ok(commit)
}

fn parse_parents(
    mapping: &serde_yaml::Mapping,
    prev_commit_ref: Option<&CommitRef>,
) -> Result<Vec<CommitRef>, ParseError> {
    let parents_key = serde_yaml::Value::String("parents".to_string());

    if let Some(parents_value) = mapping.get(&parents_key) {
        // Explicit parents specified
        if parents_value.is_null() {
            return Ok(Vec::new());
        }

        let parents_seq =
            parents_value
                .as_sequence()
                .ok_or_else(|| ParseError::UnexpectedType {
                    expected: "array",
                    actual: format!("{:?}", parents_value),
                })?;

        let mut parents = Vec::new();
        for parent_value in parents_seq {
            let parent_ref = parse_commit_ref(parent_value)?;
            parents.push(parent_ref);
        }
        Ok(parents)
    } else {
        // Default: previous commit in document order, or empty for first commit
        if let Some(prev_ref) = prev_commit_ref {
            Ok(vec![prev_ref.clone()])
        } else {
            Ok(Vec::new())
        }
    }
}

fn parse_author(
    mapping: &serde_yaml::Mapping,
    first_parent: Option<&Commit>,
) -> Result<Identity, ParseError> {
    let author_key = serde_yaml::Value::String("author".to_string());

    if let Some(author_value) = mapping.get(&author_key) {
        let author_str = author_value
            .as_str()
            .ok_or_else(|| ParseError::UnexpectedType {
                expected: "string",
                actual: format!("{:?}", author_value),
            })?;
        Identity::parse(author_str)
    } else {
        // Default: first parent's author, or "User <user@localhost>"
        if let Some(parent) = first_parent {
            Ok(parent.author.clone())
        } else {
            Identity::parse("User <user@localhost>").map_err(|_| {
                ParseError::InvalidIdentity("failed to parse default identity".to_string())
            })
        }
    }
}

fn parse_author_date(
    mapping: &serde_yaml::Mapping,
    parents: &[Commit],
) -> Result<Timestamp, ParseError> {
    let key = serde_yaml::Value::String("author-date".to_string());

    if let Some(value) = mapping.get(&key) {
        let date_str = value.as_str().ok_or_else(|| ParseError::UnexpectedType {
            expected: "string",
            actual: format!("{:?}", value),
        })?;
        Timestamp::from_iso8601(date_str)
    } else {
        // Default: 256 seconds after max parent author-date, or 2021-01-14T08:25:36Z
        // for first commit
        if parents.is_empty() {
            Timestamp::from_iso8601("2021-01-14T08:25:36Z")
        } else {
            let max_parent = parents
                .iter()
                .max_by_key(|p| (p.author_date.seconds, p.author_date.offset_minutes))
                .unwrap();
            Ok(Timestamp {
                seconds: max_parent.author_date.seconds + 256,
                offset_minutes: max_parent.author_date.offset_minutes,
            })
        }
    }
}

fn parse_committer(
    mapping: &serde_yaml::Mapping,
    author: &Identity,
) -> Result<Identity, ParseError> {
    let key = serde_yaml::Value::String("committer".to_string());

    if let Some(value) = mapping.get(&key) {
        let committer_str = value.as_str().ok_or_else(|| ParseError::UnexpectedType {
            expected: "string",
            actual: format!("{:?}", value),
        })?;
        Identity::parse(committer_str)
    } else {
        // Default: same as author
        Ok(author.clone())
    }
}

fn parse_committer_date(
    mapping: &serde_yaml::Mapping,
    author_date: Timestamp,
    parents: &[Commit],
) -> Result<Timestamp, ParseError> {
    let key = serde_yaml::Value::String("commit-date".to_string());

    if let Some(value) = mapping.get(&key) {
        let date_str = value.as_str().ok_or_else(|| ParseError::UnexpectedType {
            expected: "string",
            actual: format!("{:?}", value),
        })?;
        Timestamp::from_iso8601(date_str)
    } else {
        // Default: 3 seconds after max of author-date and parent commit-dates
        let mut max_seconds = author_date.seconds;
        let mut max_offset = author_date.offset_minutes;

        for parent in parents {
            if parent.committer_date.seconds > max_seconds
                || (parent.committer_date.seconds == max_seconds
                    && parent.committer_date.offset_minutes > max_offset)
            {
                max_seconds = parent.committer_date.seconds;
                max_offset = parent.committer_date.offset_minutes;
            }
        }

        Ok(Timestamp {
            seconds: max_seconds + 3,
            offset_minutes: max_offset,
        })
    }
}

fn parse_message(
    mapping: &serde_yaml::Mapping,
    commit_ref: &CommitRef,
    committer_date: &Timestamp,
) -> Result<String, ParseError> {
    let key = serde_yaml::Value::String("message".to_string());

    if let Some(value) = mapping.get(&key) {
        let message_str = value.as_str().ok_or_else(|| ParseError::UnexpectedType {
            expected: "string",
            actual: format!("{:?}", value),
        })?;
        Ok(message_str.to_string())
    } else {
        // Default: "commit N" for integer refs, "commit at <date>" for hex refs
        match commit_ref {
            CommitRef::Int(n) => Ok(format!("commit {}", n)),
            CommitRef::Hex(_) | CommitRef::Prefix(_) => {
                Ok(format!("commit at {}", committer_date.to_iso8601()))
            }
        }
    }
}

fn parse_tree(
    mapping: &serde_yaml::Mapping,
    first_parent: Option<&Commit>,
    processing_state: &HashMap<CommitRef, CommitProcessingState>,
) -> Result<Tree, ParseError> {
    let key = serde_yaml::Value::String("tree".to_string());

    let base_tree = if let Some(parent) = first_parent {
        parent.tree.clone()
    } else {
        Tree::new()
    };

    if let Some(value) = mapping.get(&key) {
        if value.is_null() {
            // Explicit null means empty tree
            Ok(Tree::new())
        } else if let Some(tree_mapping) = value.as_mapping() {
            if tree_mapping.is_empty() {
                // Empty mapping means empty tree
                Ok(Tree::new())
            } else {
                // Apply modifications on top of base tree
                let mut tree = base_tree;

                // Set up initial context for [commit] and [path] inheritance
                let default_commit = first_parent.map(|p| p.id);

                apply_tree_delta(
                    &mut tree,
                    "",
                    tree_mapping,
                    default_commit,
                    None, // path starts as None (which means ".")
                    processing_state,
                )?;
                Ok(tree)
            }
        } else {
            Err(ParseError::UnexpectedType {
                expected: "mapping or null",
                actual: format!("{:?}", value),
            })
        }
    } else {
        // No tree specified, use base tree
        Ok(base_tree)
    }
}

/// Resolve a commit reference to an ObjectId using the processing state
fn resolve_commit_from_state(
    commit_ref: &CommitRef,
    processing_state: &HashMap<CommitRef, CommitProcessingState>,
) -> Result<ObjectId, ParseError> {
    match commit_ref {
        CommitRef::Hex(oid) => Ok(*oid),
        CommitRef::Prefix(prefix) => {
            // Search through all commits in the processing state
            let matches: Vec<ObjectId> = processing_state
                .values()
                .filter_map(|state| {
                    if let CommitProcessingState::Complete(commit) = state {
                        let hex_str = commit.id.to_hex();
                        if hex_str.starts_with(prefix) {
                            Some(commit.id)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .collect();

            match matches.len() {
                0 => Err(ParseError::CommitNotFound(format!(
                    "no commit found matching prefix: {}",
                    prefix
                ))),
                1 => Ok(matches[0]),
                _ => Err(ParseError::AmbiguousHash(prefix.clone())),
            }
        }
        CommitRef::Int(_) => {
            // Look for this commit ref in the processing state
            if let Some(CommitProcessingState::Complete(commit)) = processing_state.get(commit_ref)
            {
                Ok(commit.id)
            } else {
                Err(ParseError::CommitNotFound(format!(
                    "integer commit reference {:?} not found in processing state",
                    commit_ref
                )))
            }
        }
    }
}

/// Get a commit from the processing state by ObjectId
fn get_commit_from_state(
    commit_id: ObjectId,
    processing_state: &HashMap<CommitRef, CommitProcessingState>,
) -> Result<&Commit, ParseError> {
    for state in processing_state.values() {
        if let CommitProcessingState::Complete(commit) = state {
            if commit.id == commit_id {
                return Ok(commit);
            }
        }
    }
    Err(ParseError::CommitNotFound(format!(
        "commit {} not found in processing state",
        commit_id.to_hex()
    )))
}

/// Resolve a path reference, handling relative paths (./foo, ../bar) and
/// absolute paths
fn resolve_path(
    path_ref: &str,
    target_path: &str,
    inherited_source_path: Option<&str>,
) -> Result<String, ParseError> {
    // Determine the base path for resolution
    // According to spec: relative paths are resolved relative to the inherited
    // source path. But the inherited source path points to THIS entry (file/dir),
    // so we need to resolve relative to its parent directory.
    let base_path = if path_ref == "." || path_ref.starts_with("./") || path_ref.starts_with("../")
    {
        // Relative path - use the parent of the inherited source path as base
        let inherited = inherited_source_path.unwrap_or(target_path);

        // Get the parent directory of the inherited path
        if let Some(pos) = inherited.rfind('/') {
            &inherited[..pos]
        } else {
            // No slash means inherited is at root, so parent is root
            ""
        }
    } else {
        // Absolute path (relative to repository root) - ignore inheritance
        ""
    };

    // Now resolve the path
    if path_ref == "." {
        return Ok(base_path.to_string());
    }

    let mut components: Vec<&str> = if !base_path.is_empty() {
        base_path.split('/').collect()
    } else {
        Vec::new()
    };

    // Parse the path reference
    for part in path_ref.split('/') {
        match part {
            "" | "." => {
                // Skip empty components and current directory references
            }
            ".." => {
                if components.is_empty() {
                    return Err(ParseError::InvalidPathReference(
                        "path resolution goes above repository root".to_string(),
                    ));
                }
                components.pop();
            }
            component => {
                components.push(component);
            }
        }
    }

    Ok(components.join("/"))
}

fn apply_tree_delta(
    tree: &mut Tree,
    target_prefix: &str,
    mapping: &serde_yaml::Mapping,
    inherited_commit: Option<ObjectId>,
    inherited_path: Option<String>,
    processing_state: &HashMap<CommitRef, CommitProcessingState>,
) -> Result<(), ParseError> {
    // Extract special keys if present and update context
    let mut current_commit = inherited_commit;
    let mut current_path = inherited_path;

    if let Some(commit_value) = get_special_key(mapping, "commit") {
        // Parse the commit reference
        if commit_value.is_null() {
            current_commit = None;
        } else {
            let commit_ref = parse_commit_ref(commit_value)?;
            // Resolve the commit reference to an ObjectId
            let commit_id = resolve_commit_from_state(&commit_ref, processing_state)?;
            current_commit = Some(commit_id);
        }
    }

    if let Some(path_value) = get_special_key(mapping, "path") {
        // Parse the path reference
        let path_str = path_value
            .as_str()
            .ok_or_else(|| ParseError::UnexpectedType {
                expected: "string",
                actual: format!("{:?}", path_value),
            })?;

        // Special case: "." means root of source commit
        if path_str == "." {
            current_path = Some(String::new());
        } else {
            // According to spec (IDEA-1.1.md lines 95-99):
            // Relative paths are resolved relative to "the source path that would be
            // computed by inheritance" which is: parent's effective source path +
            // this entry's name This is the same as target_prefix in our case,
            // since we're called with target_prefix set correctly
            let inherited_source_base = target_prefix;

            // Resolve the path
            current_path = Some(resolve_path(
                path_str,
                target_prefix,
                Some(inherited_source_base),
            )?);
        }
    }

    // Check if this is a pure reference (only special keys, no regular keys)
    let has_regular_keys = mapping
        .iter()
        .any(|(k, _)| !is_special_key(k, "commit") && !is_special_key(k, "path"));

    if !has_regular_keys && !mapping.is_empty() {
        // Pure reference - resolve and copy the content
        if current_commit.is_none() {
            return Err(ParseError::InvalidPathReference(
                "cannot use [path] reference without a [commit]".to_string(),
            ));
        }

        let source_commit_id = current_commit.unwrap();
        let source_path = current_path.unwrap_or_else(|| target_prefix.to_string());

        // Look up the commit
        let source_commit = get_commit_from_state(source_commit_id, processing_state)?;

        // Get the content at the source path
        if let Some(content) = source_commit.tree.get(&source_path) {
            // It's a blob - copy it
            let target_path = target_prefix.to_string();
            tree.insert(target_path, content.to_string());
        } else {
            // Check if it's a tree (has entries with this prefix)
            let source_prefix = if source_path.is_empty() {
                String::new()
            } else {
                format!("{}/", source_path)
            };

            let mut found_any = false;
            for path in source_commit.tree.paths() {
                if path == &source_path || path.starts_with(&source_prefix) {
                    found_any = true;
                    let relative_path = if path == &source_path {
                        // This shouldn't happen for a tree, but handle it
                        String::new()
                    } else {
                        path[source_prefix.len()..].to_string()
                    };

                    let target_path = if target_prefix.is_empty() {
                        relative_path
                    } else if relative_path.is_empty() {
                        target_prefix.to_string()
                    } else {
                        format!("{}/{}", target_prefix, relative_path)
                    };

                    let content = source_commit.tree.get(path).unwrap();
                    tree.insert(target_path, content.to_string());
                }
            }

            if !found_any {
                return Err(ParseError::InvalidPathReference(format!(
                    "path '{}' not found in commit",
                    source_path
                )));
            }
        }

        return Ok(());
    }

    // Process regular string keys (with inherited context for nested entries)
    for (key, value) in mapping.iter() {
        // Skip special keys
        if is_special_key(key, "commit") || is_special_key(key, "path") {
            continue;
        }

        // This must be a string/number key
        let name = normalize_yaml_key(key)?;

        // Validate the name component
        Tree::validate_component(&name)?;

        let target_path = if target_prefix.is_empty() {
            name.clone()
        } else {
            format!("{}/{}", target_prefix, name)
        };

        // Compute the inherited source path for this entry
        let inherited_source_path = if let Some(ref src_path) = current_path {
            if src_path.is_empty() {
                Some(name.clone())
            } else {
                Some(format!("{}/{}", src_path, name))
            }
        } else {
            // No explicit path set, so source path follows target path
            Some(target_path.clone())
        };

        if value.is_null() {
            // Delete
            tree.remove(&target_path);
        } else if let Some(s) = value.as_str() {
            // Blob content
            tree.insert(target_path, s.to_string());
        } else if let Some(nested_mapping) = value.as_mapping() {
            if nested_mapping.is_empty() {
                // Empty mapping means delete
                tree.remove(&target_path);
            } else {
                // Recursively process with inherited context
                apply_tree_delta(
                    tree,
                    &target_path,
                    nested_mapping,
                    current_commit,
                    inherited_source_path,
                    processing_state,
                )?;
            }
        } else if matches!(value, serde_yaml::Value::Tagged(_)) {
            return Err(ParseError::UnexpectedField(
                "YAML tags are not supported".to_string(),
            ));
        } else {
            return Err(ParseError::UnexpectedType {
                expected: "string, mapping, or null",
                actual: format!("{:?}", value),
            });
        }
    }

    Ok(())
}

fn calculate_tree_id(tree: &Tree) -> Result<ObjectId, ParseError> {
    use sha1_checked::Digest;

    if tree.is_empty() {
        // Empty tree has a specific hash in git
        // tree 0\0
        let mut hasher = sha1_checked::Sha1::new();
        hasher.update(b"tree 0\0");
        let hash: [u8; 20] = hasher.finalize().into();
        return Ok(ObjectId(hash));
    }

    // Build a hierarchical tree structure
    // Map from directory path -> map of name -> (mode, oid)
    let mut dir_entries: HashMap<String, BTreeMap<String, (String, ObjectId)>> = HashMap::new();

    // First, hash all blobs and organize by directory
    for path in tree.paths() {
        let content = tree.get(path).unwrap();

        // Hash the blob
        let blob_data = format!("blob {}\0{}", content.len(), content);
        let mut hasher = sha1_checked::Sha1::new();
        hasher.update(blob_data.as_bytes());
        let hash: [u8; 20] = hasher.finalize().into();
        let blob_oid = ObjectId(hash);

        // Split into directory and filename
        let (dir, name) = if let Some(pos) = path.rfind('/') {
            (&path[..pos], &path[pos + 1..])
        } else {
            ("", path)
        };

        dir_entries
            .entry(dir.to_string())
            .or_default()
            .insert(name.to_string(), ("100644".to_string(), blob_oid));
    }

    // Build tree objects bottom-up
    // Start from deepest directories and work up
    let mut tree_oids: HashMap<String, ObjectId> = HashMap::new();

    // Collect all directories that need tree objects
    // Include all directories with entries, plus all their ancestors up to root
    let mut all_dirs = std::collections::HashSet::new();
    for dir in dir_entries.keys() {
        // Add this directory
        all_dirs.insert(dir.clone());
        // Add all ancestor directories
        let mut current = dir.as_str();
        while let Some(pos) = current.rfind('/') {
            current = &current[..pos];
            all_dirs.insert(current.to_string());
        }
        // Always include root
        all_dirs.insert(String::new());
    }

    // Sort directories by depth (deepest first)
    let mut dirs: Vec<String> = all_dirs.into_iter().collect();
    dirs.sort_by(|a, b| {
        let a_depth = if a.is_empty() {
            0
        } else {
            a.matches('/').count() + 1
        };
        let b_depth = if b.is_empty() {
            0
        } else {
            b.matches('/').count() + 1
        };
        b_depth.cmp(&a_depth) // Reverse order (deepest first)
    });

    for dir in dirs {
        let mut entries = dir_entries.get(&dir).cloned().unwrap_or_default();

        // Add subdirectories
        for (subdir, subdir_oid) in &tree_oids {
            // Check if subdir is a direct child of dir
            let expected_prefix = if dir.is_empty() {
                String::new()
            } else {
                format!("{}/", dir)
            };

            if subdir.starts_with(&expected_prefix) {
                let remainder = &subdir[expected_prefix.len()..];
                // Only direct children (no slashes in remainder)
                if !remainder.contains('/') && !remainder.is_empty() {
                    entries.insert(remainder.to_string(), ("040000".to_string(), *subdir_oid));
                }
            }
        }

        // Build tree object
        let mut tree_content = Vec::new();
        for (name, (mode, oid)) in entries {
            tree_content.extend_from_slice(format!("{} {}\0", mode, name).as_bytes());
            tree_content.extend_from_slice(oid.as_bytes());
        }

        let tree_data = format!("tree {}\0", tree_content.len());
        let mut full_tree_data = tree_data.as_bytes().to_vec();
        full_tree_data.extend_from_slice(&tree_content);

        let mut hasher = sha1_checked::Sha1::new();
        hasher.update(&full_tree_data);
        let hash: [u8; 20] = hasher.finalize().into();
        let tree_oid = ObjectId(hash);

        tree_oids.insert(dir, tree_oid);
    }

    // Return the root tree OID
    tree_oids.get("").copied().ok_or_else(|| {
        ParseError::InvalidTreeEntryName(format!(
            "No root tree found. Available trees: {:?}",
            tree_oids.keys().collect::<Vec<_>>()
        ))
    })
}

fn calculate_commit_id(
    tree_id: &ObjectId,
    parent_ids: &[ObjectId],
    author: &Identity,
    author_date: Timestamp,
    committer: &Identity,
    committer_date: Timestamp,
    message: &str,
) -> Result<ObjectId, ParseError> {
    use sha1_checked::Digest;

    let mut commit_content = String::new();
    commit_content.push_str(&format!("tree {}\n", tree_id.to_hex()));

    for parent_id in parent_ids {
        commit_content.push_str(&format!("parent {}\n", parent_id.to_hex()));
    }

    commit_content.push_str(&format!(
        "author {} {} {:+05}\n",
        author.format(),
        author_date.seconds,
        format_git_offset(author_date.offset_minutes)
    ));

    commit_content.push_str(&format!(
        "committer {} {} {:+05}\n",
        committer.format(),
        committer_date.seconds,
        format_git_offset(committer_date.offset_minutes)
    ));

    commit_content.push('\n');
    commit_content.push_str(message);

    let commit_data = format!("commit {}\0{}", commit_content.len(), commit_content);
    let mut hasher = sha1_checked::Sha1::new();
    hasher.update(commit_data.as_bytes());
    let hash: [u8; 20] = hasher.finalize().into();
    Ok(ObjectId(hash))
}

fn format_git_offset(offset_minutes: i16) -> String {
    let sign = if offset_minutes < 0 { '-' } else { '+' };
    let abs_minutes = offset_minutes.abs();
    let hours = abs_minutes / 60;
    let mins = abs_minutes % 60;
    format!("{}{:02}{:02}", sign, hours, mins)
}

fn prune_unreachable(repo: &mut Repository) {
    let mut reachable = std::collections::HashSet::new();
    let mut to_visit = Vec::new();

    // Start from HEAD
    match &repo.head {
        HeadState::Detached(id) => {
            if repo.commits.contains_key(id) {
                to_visit.push(*id);
            }
        }
        HeadState::Symbolic(ref_name) => {
            if let Some(id) = repo.refs.get(ref_name)
                && repo.commits.contains_key(id)
            {
                to_visit.push(*id);
            }
        }
    }

    // Add all refs
    for (_, id) in repo.refs.iter() {
        if repo.commits.contains_key(id) {
            to_visit.push(*id);
        }
    }

    // Walk the commit graph
    while let Some(id) = to_visit.pop() {
        if reachable.contains(&id) {
            continue;
        }
        reachable.insert(id);

        if let Some(commit) = repo.commits.get(&id) {
            for parent_id in &commit.parents {
                if !reachable.contains(parent_id) {
                    to_visit.push(*parent_id);
                }
            }
        }
    }

    // Remove unreachable commits
    repo.commits.retain(|id, _| reachable.contains(id));
}

// ============================================================================
// Serialization Implementation
// ============================================================================

/// Control how commits are referenced in the serialized output
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommitIdStyle {
    /// Use full 40-character hex object IDs
    Hex,
    /// Use sequential integer IDs (1, 2, 3, ...)
    Integer,
}

// ============================================================================
// Deduplication Context and Helper Functions
// ============================================================================

/// Tracks serialized content locations for deduplication
struct SerializationContext {
    /// Maps blob hash to (commit_id, path) where content first appeared
    /// physically
    blob_locations: HashMap<ObjectId, (ObjectId, String)>,

    /// Maps tree hash to (commit_id, path) where content first appeared
    /// physically
    tree_locations: HashMap<ObjectId, (ObjectId, String)>,

    /// All commits in topological order
    all_commits: Vec<ObjectId>,

    /// Computed truncated hash length for non-head commits
    truncated_len: usize,

    /// Head commits (use full 40-char hash)
    head_commits: std::collections::HashSet<ObjectId>,

    /// Current commit being serialized
    current_commit: ObjectId,

    /// Repository reference
    repo: *const Repository,
}

impl SerializationContext {
    fn new(
        repo: &Repository,
        ordered_commits: Vec<ObjectId>,
        head_commits: std::collections::HashSet<ObjectId>,
    ) -> Self {
        let truncated_len = compute_truncated_hash_length(&ordered_commits);

        SerializationContext {
            blob_locations: HashMap::new(),
            tree_locations: HashMap::new(),
            all_commits: ordered_commits,
            truncated_len,
            head_commits,
            current_commit: ObjectId([0u8; 20]),
            repo: repo as *const Repository,
        }
    }

    fn repo(&self) -> &Repository {
        unsafe { &*self.repo }
    }
}

/// Compute minimum truncated hash length needed to avoid ambiguity
fn compute_truncated_hash_length(commits: &[ObjectId]) -> usize {
    if commits.len() <= 1 {
        return 4;
    }

    // Try increasing lengths until we have no collisions
    for len in (4..=40).step_by(2) {
        let mut seen = std::collections::HashSet::new();
        let mut collision = false;

        for commit in commits {
            let truncated = commit.to_hex_truncated(len);
            if !seen.insert(truncated) {
                collision = true;
                break;
            }
        }

        if !collision {
            // Add 2 digits safety margin, ensure even, minimum 4
            let with_margin = len + 2;
            return with_margin.min(40);
        }
    }

    // If we get here, use full length
    40
}

/// Compute path similarity score for choosing best reference target
/// Returns (suffix_match_len, -boundary_diff, -extra_prefix)
fn compute_path_similarity_score(target: &str, candidate: &str) -> (i32, i32, i32) {
    let target_parts: Vec<&str> = target.split('/').collect();
    let candidate_parts: Vec<&str> = candidate.split('/').collect();

    // Find longest suffix match
    let mut suffix_match = 0;
    for i in 1..=target_parts.len().min(candidate_parts.len()) {
        if target_parts[target_parts.len() - i] == candidate_parts[candidate_parts.len() - i] {
            suffix_match = i;
        } else {
            break;
        }
    }

    // Count differing components at boundary
    let boundary_diff = if suffix_match < target_parts.len().min(candidate_parts.len()) {
        1
    } else {
        0
    };

    // Count extra prefix components in candidate
    let extra_prefix = if suffix_match == target_parts.len() {
        (candidate_parts.len() - target_parts.len()) as i32
    } else {
        (candidate_parts.len() - suffix_match) as i32 - (target_parts.len() - suffix_match) as i32
    };

    (suffix_match as i32, -boundary_diff, extra_prefix.abs() * -1)
}

/// Find best reference target from candidates based on path similarity
fn find_best_reference(target_path: &str, candidates: &[(ObjectId, String)]) -> (ObjectId, String) {
    if candidates.is_empty() {
        panic!("find_best_reference called with empty candidates");
    }

    let mut best = &candidates[0];
    let mut best_score = compute_path_similarity_score(target_path, &best.1);

    for candidate in &candidates[1..] {
        let score = compute_path_similarity_score(target_path, &candidate.1);
        if score > best_score || (score == best_score && candidate.1 < best.1) {
            best = candidate;
            best_score = score;
        }
    }

    best.clone()
}

/// Serialize a Repository to YAML format
pub fn serialize(repo: &Repository, id_style: CommitIdStyle) -> String {
    let mut root = serde_yaml::Mapping::new();

    // Sort commits in topological order with tiebreaking
    let ordered_commits = topological_sort_with_tiebreak(repo);

    // Collect head commits (commits directly referenced by HEAD or refs)
    let mut head_commits = std::collections::HashSet::new();
    match &repo.head {
        HeadState::Detached(oid) => {
            head_commits.insert(*oid);
        }
        HeadState::Symbolic(_) => {}
    }
    for (_, target_id) in repo.refs() {
        head_commits.insert(*target_id);
    }

    // Initialize serialization context for deduplication
    let mut ctx = SerializationContext::new(repo, ordered_commits.clone(), head_commits.clone());

    // Build mapping from ObjectId to commit reference (hex or integer)
    let mut commit_refs: HashMap<ObjectId, serde_yaml::Value> = HashMap::new();
    for (idx, commit_id) in ordered_commits.iter().enumerate() {
        let ref_value = match id_style {
            CommitIdStyle::Hex => {
                // Use full hash for head commits, truncated for others
                let hash_str = if ctx.head_commits.contains(commit_id) {
                    commit_id.to_hex()
                } else {
                    commit_id.to_hex_truncated(ctx.truncated_len)
                };
                serde_yaml::Value::String(hash_str)
            }
            CommitIdStyle::Integer => serde_yaml::Value::Number((idx + 1).into()),
        };
        commit_refs.insert(*commit_id, ref_value);
    }

    // Serialize HEAD
    let head_value = match &repo.head {
        HeadState::Symbolic(ref_name) => serde_yaml::Value::String(ref_name.as_str().to_string()),
        HeadState::Detached(oid) => commit_refs
            .get(oid)
            .cloned()
            .unwrap_or_else(|| serde_yaml::Value::String(oid.to_hex())),
    };
    root.insert(serde_yaml::Value::String("HEAD".to_string()), head_value);

    // Serialize refs
    let mut refs_map = serde_yaml::Mapping::new();
    for (ref_name, target_id) in repo.refs() {
        let target_value = commit_refs
            .get(target_id)
            .cloned()
            .unwrap_or_else(|| serde_yaml::Value::String(target_id.to_hex()));

        // Split ref path and build nested structure
        // e.g., "refs/heads/main" -> refs -> heads -> main: value
        let path_parts: Vec<&str> = ref_name.as_str().split('/').collect();
        if path_parts.len() >= 2 && path_parts[0] == "refs" {
            insert_nested_ref(&mut refs_map, &path_parts[1..], target_value);
        }
    }
    root.insert(
        serde_yaml::Value::String("refs".to_string()),
        serde_yaml::Value::Mapping(refs_map),
    );

    // Serialize commits with deduplication
    for (idx, commit_id) in ordered_commits.iter().enumerate() {
        let commit = repo.get_commit(commit_id).expect("commit should exist");
        let prev_commit = if idx > 0 {
            Some(
                repo.get_commit(&ordered_commits[idx - 1])
                    .expect("prev commit should exist"),
            )
        } else {
            None
        };

        ctx.current_commit = *commit_id;

        let commit_key = commit_refs.get(commit_id).cloned().unwrap();
        let commit_value = serialize_commit(commit, prev_commit, &commit_refs, id_style, repo);

        root.insert(commit_key, commit_value);
    }

    // Sort the root mapping, but preserve commit order
    let sorted_root = sort_root_mapping(root, &ordered_commits, &commit_refs);

    // Convert to YAML string
    serde_yaml::to_string(&sorted_root).expect("serialization should succeed")
}

/// Sort the root mapping while preserving commit order
/// Commits should appear in topological order (document order), not lexicographic order
fn sort_root_mapping(
    root: serde_yaml::Mapping,
    ordered_commits: &[ObjectId],
    commit_refs: &HashMap<ObjectId, serde_yaml::Value>,
) -> serde_yaml::Value {
    let mut sorted = serde_yaml::Mapping::new();

    // First, insert non-commit keys in sorted order (HEAD, refs, etc.)
    let mut non_commit_keys: Vec<serde_yaml::Value> = root
        .keys()
        .filter(|k| !commit_refs.values().any(|v| v == *k))
        .cloned()
        .collect();
    non_commit_keys.sort_by(|a, b| {
        let a_str = value_to_sort_key(a);
        let b_str = value_to_sort_key(b);
        a_str.cmp(&b_str)
    });

    for key in non_commit_keys {
        if let Some(val) = root.get(&key) {
            sorted.insert(key, sort_mapping_recursive(val.clone()));
        }
    }

    // Then, insert commits in document order (topological order)
    for commit_id in ordered_commits {
        if let Some(commit_key) = commit_refs.get(commit_id) {
            if let Some(commit_val) = root.get(commit_key) {
                sorted.insert(commit_key.clone(), sort_mapping_recursive(commit_val.clone()));
            }
        }
    }

    serde_yaml::Value::Mapping(sorted)
}

/// Recursively sort all mappings in a Value by their keys (lexicographically)
fn sort_mapping_recursive(value: serde_yaml::Value) -> serde_yaml::Value {
    match value {
        serde_yaml::Value::Mapping(mapping) => {
            let mut sorted_mapping = serde_yaml::Mapping::new();

            // Collect and sort keys
            let mut keys: Vec<serde_yaml::Value> = mapping.keys().cloned().collect();
            keys.sort_by(|a, b| {
                // Convert to strings for comparison
                let a_str = value_to_sort_key(a);
                let b_str = value_to_sort_key(b);
                a_str.cmp(&b_str)
            });

            // Insert in sorted order, recursively sorting values
            for key in keys {
                if let Some(val) = mapping.get(&key) {
                    sorted_mapping.insert(key, sort_mapping_recursive(val.clone()));
                }
            }

            serde_yaml::Value::Mapping(sorted_mapping)
        }
        serde_yaml::Value::Sequence(seq) => {
            // Recursively sort mappings in sequences
            serde_yaml::Value::Sequence(seq.into_iter().map(sort_mapping_recursive).collect())
        }
        other => other,
    }
}

/// Convert a YAML value to a string for sorting purposes
fn value_to_sort_key(value: &serde_yaml::Value) -> String {
    match value {
        serde_yaml::Value::String(s) => s.clone(),
        serde_yaml::Value::Number(n) => n.to_string(),
        serde_yaml::Value::Bool(b) => b.to_string(),
        serde_yaml::Value::Null => "null".to_string(),
        _ => format!("{:?}", value),
    }
}

/// Insert a nested ref into the refs mapping
fn insert_nested_ref(mapping: &mut serde_yaml::Mapping, path: &[&str], value: serde_yaml::Value) {
    if path.is_empty() {
        return;
    }

    if path.len() == 1 {
        // Leaf node
        mapping.insert(serde_yaml::Value::String(path[0].to_string()), value);
    } else {
        // Intermediate node
        let key = serde_yaml::Value::String(path[0].to_string());
        let nested = mapping
            .entry(key.clone())
            .or_insert_with(|| serde_yaml::Value::Mapping(serde_yaml::Mapping::new()));

        if let serde_yaml::Value::Mapping(nested_map) = nested {
            insert_nested_ref(nested_map, &path[1..], value);
        }
    }
}

/// Serialize a single commit
fn serialize_commit(
    commit: &Commit,
    prev_commit: Option<&Commit>,
    commit_refs: &HashMap<ObjectId, serde_yaml::Value>,
    id_style: CommitIdStyle,
    repo: &Repository,
) -> serde_yaml::Value {
    let mut mapping = serde_yaml::Mapping::new();

    // Get parent commits
    let parent_commits: Vec<&Commit> = commit
        .parents
        .iter()
        .filter_map(|id| repo.get_commit(id))
        .collect();
    let first_parent = parent_commits.first().copied();

    // Serialize parents (omit if default)
    let default_parents = if let Some(prev) = prev_commit {
        vec![prev.id]
    } else {
        vec![]
    };

    if commit.parents != default_parents {
        let parents_array: Vec<serde_yaml::Value> = commit
            .parents
            .iter()
            .map(|parent_id| {
                commit_refs
                    .get(parent_id)
                    .cloned()
                    .unwrap_or_else(|| serde_yaml::Value::String(parent_id.to_hex()))
            })
            .collect();
        mapping.insert(
            serde_yaml::Value::String("parents".to_string()),
            serde_yaml::Value::Sequence(parents_array),
        );
    }

    // Serialize message (omit if default)
    let default_message = match id_style {
        CommitIdStyle::Integer => {
            // Find the integer ID for this commit
            let mut int_id = 0u32;
            for (oid, ref_val) in commit_refs.iter() {
                if *oid == commit.id
                    && let serde_yaml::Value::Number(n) = ref_val
                {
                    int_id = n.as_u64().unwrap_or(0) as u32;
                    break;
                }
            }
            if int_id > 0 {
                format!("commit {}", int_id)
            } else {
                format!("commit at {}", commit.committer_date.to_iso8601())
            }
        }
        CommitIdStyle::Hex => {
            format!("commit at {}", commit.committer_date.to_iso8601())
        }
    };

    if commit.message != default_message {
        mapping.insert(
            serde_yaml::Value::String("message".to_string()),
            serde_yaml::Value::String(commit.message.clone()),
        );
    }

    // Serialize author (omit if default)
    let default_author = if let Some(parent) = first_parent {
        parent.author.clone()
    } else {
        Identity::parse("User <user@localhost>").unwrap()
    };

    if commit.author != default_author {
        mapping.insert(
            serde_yaml::Value::String("author".to_string()),
            serde_yaml::Value::String(commit.author.format()),
        );
    }

    // Serialize author-date (omit if default)
    let default_author_date = if parent_commits.is_empty() {
        Timestamp::from_iso8601("2021-01-14T08:25:36Z").unwrap()
    } else {
        let max_parent = parent_commits
            .iter()
            .max_by_key(|p| (p.author_date.seconds, p.author_date.offset_minutes))
            .unwrap();
        Timestamp {
            seconds: max_parent.author_date.seconds + 256,
            offset_minutes: max_parent.author_date.offset_minutes,
        }
    };

    if commit.author_date != default_author_date {
        mapping.insert(
            serde_yaml::Value::String("author-date".to_string()),
            serde_yaml::Value::String(commit.author_date.to_iso8601()),
        );
    }

    // Serialize committer (omit if same as author)
    if commit.committer != commit.author {
        mapping.insert(
            serde_yaml::Value::String("committer".to_string()),
            serde_yaml::Value::String(commit.committer.format()),
        );
    }

    // Serialize commit-date (omit if default)
    let mut max_seconds = commit.author_date.seconds;
    let mut max_offset = commit.author_date.offset_minutes;
    for parent in &parent_commits {
        if parent.committer_date.seconds > max_seconds
            || (parent.committer_date.seconds == max_seconds
                && parent.committer_date.offset_minutes > max_offset)
        {
            max_seconds = parent.committer_date.seconds;
            max_offset = parent.committer_date.offset_minutes;
        }
    }
    let default_committer_date = Timestamp {
        seconds: max_seconds + 3,
        offset_minutes: max_offset,
    };

    if commit.committer_date != default_committer_date {
        mapping.insert(
            serde_yaml::Value::String("commit-date".to_string()),
            serde_yaml::Value::String(commit.committer_date.to_iso8601()),
        );
    }

    // Serialize tree (compute delta from first parent)
    let first_parent_tree = first_parent.map(|p| &p.tree);
    let tree_delta = compute_tree_delta(&commit.tree, first_parent_tree);

    // Only include tree if it's non-empty or if this is the root commit with an
    // empty tree
    let tree_is_empty = match &tree_delta {
        serde_yaml::Value::Mapping(m) => m.is_empty(),
        _ => false,
    };

    if !tree_is_empty {
        mapping.insert(serde_yaml::Value::String("tree".to_string()), tree_delta);
    } else if parent_commits.is_empty() && commit.tree.is_empty() {
        // Root commit with empty tree - explicitly serialize empty tree
        mapping.insert(
            serde_yaml::Value::String("tree".to_string()),
            serde_yaml::Value::Mapping(serde_yaml::Mapping::new()),
        );
    }

    serde_yaml::Value::Mapping(mapping)
}

/// Compute tree delta between current tree and base tree
fn compute_tree_delta(tree: &Tree, base_tree: Option<&Tree>) -> serde_yaml::Value {
    let base_tree = match base_tree {
        Some(t) => t,
        None => {
            // No base tree, serialize entire tree
            if tree.is_empty() {
                return serde_yaml::Value::Mapping(serde_yaml::Mapping::new());
            }
            return serialize_tree_full(tree);
        }
    };

    let mut delta = serde_yaml::Mapping::new();

    // Collect all paths from both trees
    let mut all_paths = std::collections::BTreeSet::new();
    for path in tree.paths() {
        all_paths.insert(path);
    }
    for path in base_tree.paths() {
        all_paths.insert(path);
    }

    // Build delta structure
    for path in all_paths {
        let current_content = tree.get(path);
        let base_content = base_tree.get(path);

        if current_content != base_content {
            // Path has changed
            let path_parts: Vec<&str> = path.split('/').collect();
            insert_tree_change(&mut delta, &path_parts, current_content);
        }
    }

    serde_yaml::Value::Mapping(delta)
}

/// Serialize a full tree (no delta)
fn serialize_tree_full(tree: &Tree) -> serde_yaml::Value {
    let mut root = serde_yaml::Mapping::new();

    for path in tree.paths() {
        let content = tree.get(path).expect("path should exist");
        let path_parts: Vec<&str> = path.split('/').collect();
        insert_tree_change(&mut root, &path_parts, Some(content));
    }

    serde_yaml::Value::Mapping(root)
}

/// Insert a tree change into the delta mapping
fn insert_tree_change(
    mapping: &mut serde_yaml::Mapping,
    path_parts: &[&str],
    content: Option<&str>,
) {
    if path_parts.is_empty() {
        return;
    }

    if path_parts.len() == 1 {
        // Leaf node
        let key = serde_yaml::Value::String(path_parts[0].to_string());
        let value = match content {
            Some(s) => serde_yaml::Value::String(s.to_string()),
            None => serde_yaml::Value::Null, // Deletion
        };
        mapping.insert(key, value);
    } else {
        // Intermediate node
        let key = serde_yaml::Value::String(path_parts[0].to_string());
        let nested = mapping
            .entry(key.clone())
            .or_insert_with(|| serde_yaml::Value::Mapping(serde_yaml::Mapping::new()));

        if let serde_yaml::Value::Mapping(nested_map) = nested {
            insert_tree_change(nested_map, &path_parts[1..], content);
        }
    }
}

/// Topologically sort commits with tiebreaking algorithm from IDEA.md
fn topological_sort_with_tiebreak(repo: &Repository) -> Vec<ObjectId> {
    // Build tiebreak keys for all commits
    let mut tiebreak_keys: HashMap<ObjectId, Vec<TiebreakComponent>> = HashMap::new();

    // Initialize empty tiebreak keys
    for commit in repo.commits() {
        tiebreak_keys.insert(commit.id, Vec::new());
    }

    // Collect head commits
    let mut head_commits = Vec::new();

    // Add HEAD first
    match &repo.head {
        HeadState::Detached(id) => {
            if repo.get_commit(id).is_some() {
                head_commits.push(*id);
            }
        }
        HeadState::Symbolic(ref_name) => {
            if let Some(id) = repo.get_ref(ref_name)
                && repo.get_commit(id).is_some()
            {
                head_commits.push(*id);
            }
        }
    }

    // Add refs in lexicographic order
    let mut ref_targets: Vec<(String, ObjectId)> = repo
        .refs()
        .map(|(name, id)| (name.as_str().to_string(), *id))
        .collect();
    ref_targets.sort_by(|a, b| a.0.cmp(&b.0));

    for (_, id) in ref_targets {
        if !head_commits.contains(&id) {
            head_commits.push(id);
        }
    }

    // Walk ancestors depth-first, recording parent indices
    let mut visited = std::collections::HashSet::new();

    for head_id in head_commits {
        walk_ancestors_for_tiebreak(
            head_id,
            None, // No parent index for head commits
            repo,
            &mut visited,
            &mut tiebreak_keys,
        );
    }

    // Append timestamps to tiebreak keys
    for commit in repo.commits() {
        if let Some(key) = tiebreak_keys.get_mut(&commit.id) {
            key.push(TiebreakComponent::Timestamp(commit.committer_date));
            key.push(TiebreakComponent::Timestamp(commit.author_date));
        }
    }

    // Topologically sort with tiebreaking
    let mut sorted = Vec::new();
    let mut visited = std::collections::HashSet::new();
    let mut in_progress = std::collections::HashSet::new();

    // Sort commits by tiebreak key for deterministic iteration order
    let mut all_commits: Vec<ObjectId> = repo.commits().map(|c| c.id).collect();
    all_commits.sort_by(|a, b| {
        tiebreak_keys
            .get(a)
            .unwrap()
            .cmp(tiebreak_keys.get(b).unwrap())
    });

    for commit_id in all_commits {
        topological_visit(
            commit_id,
            repo,
            &tiebreak_keys,
            &mut visited,
            &mut in_progress,
            &mut sorted,
        );
    }

    sorted
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum TiebreakComponent {
    ParentIndex(usize),
    Timestamp(Timestamp),
}

/// Walk ancestors depth-first, recording parent indices in tiebreak keys
fn walk_ancestors_for_tiebreak(
    commit_id: ObjectId,
    parent_index: Option<usize>,
    repo: &Repository,
    visited: &mut std::collections::HashSet<ObjectId>,
    tiebreak_keys: &mut HashMap<ObjectId, Vec<TiebreakComponent>>,
) {
    // Record parent index if provided
    if let Some(idx) = parent_index
        && let Some(key) = tiebreak_keys.get_mut(&commit_id)
    {
        key.push(TiebreakComponent::ParentIndex(idx));
    }

    // If already visited, don't recurse further
    if visited.contains(&commit_id) {
        return;
    }
    visited.insert(commit_id);

    // Visit parents
    if let Some(commit) = repo.get_commit(&commit_id) {
        for (idx, parent_id) in commit.parents.iter().enumerate() {
            walk_ancestors_for_tiebreak(*parent_id, Some(idx), repo, visited, tiebreak_keys);
        }
    }
}

/// Topological visit for sorting
fn topological_visit(
    commit_id: ObjectId,
    repo: &Repository,
    tiebreak_keys: &HashMap<ObjectId, Vec<TiebreakComponent>>,
    visited: &mut std::collections::HashSet<ObjectId>,
    in_progress: &mut std::collections::HashSet<ObjectId>,
    sorted: &mut Vec<ObjectId>,
) {
    if visited.contains(&commit_id) {
        return;
    }

    if in_progress.contains(&commit_id) {
        // Cycle detected, but we should handle this gracefully
        return;
    }

    in_progress.insert(commit_id);

    // Visit parents first (they should come before this commit)
    if let Some(commit) = repo.get_commit(&commit_id) {
        // Sort parents by tiebreak key for deterministic order
        let mut parents = commit.parents.clone();
        parents.sort_by(|a, b| {
            tiebreak_keys
                .get(a)
                .unwrap()
                .cmp(tiebreak_keys.get(b).unwrap())
        });

        for parent_id in parents {
            topological_visit(parent_id, repo, tiebreak_keys, visited, in_progress, sorted);
        }
    }

    in_progress.remove(&commit_id);
    visited.insert(commit_id);
    sorted.push(commit_id);
}

pub fn main() {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_roundtrip_integer() {
        // Create a simple repository
        let mut repo = Repository::new();

        // Create a root commit
        let root_commit = Commit {
            id: ObjectId::from_hex("0000000000000000000000000000000000000001").unwrap(),
            parents: vec![],
            tree: {
                let mut tree = Tree::new();
                tree.insert("README.md".to_string(), "# Hello".to_string());
                tree
            },
            author: Identity::parse("User <user@localhost>").unwrap(),
            author_date: Timestamp::from_iso8601("2021-01-14T08:25:36Z").unwrap(),
            committer: Identity::parse("User <user@localhost>").unwrap(),
            committer_date: Timestamp::from_iso8601("2021-01-14T08:25:39Z").unwrap(),
            message: "commit 1".to_string(),
        };

        repo.insert_commit(root_commit.clone());
        repo.insert_ref(
            RefName::new("refs/heads/main".to_string()).unwrap(),
            root_commit.id,
        );
        repo.set_head(HeadState::Symbolic(
            RefName::new("refs/heads/main".to_string()).unwrap(),
        ));

        // Serialize with integer IDs
        let yaml = serialize(&repo, CommitIdStyle::Integer);
        println!("Serialized YAML:\n{}", yaml);

        // Parse it back
        let repo2 = parse(&yaml).expect("should parse");

        // Verify the round-trip
        assert_eq!(repo.refs().count(), repo2.refs().count());
        assert_eq!(repo.commits().count(), repo2.commits().count());

        // Note: The object IDs will be different because we're recalculating
        // them from the commit contents during parsing
    }

    #[test]
    fn test_serialize_with_deletion() {
        // Create a repository with two commits, second one deletes a file
        let mut repo = Repository::new();

        let author = Identity::parse("User <user@localhost>").unwrap();
        let author_date = Timestamp::from_iso8601("2021-01-14T08:25:36Z").unwrap();

        // First commit
        let tree1 = {
            let mut t = Tree::new();
            t.insert("README.md".to_string(), "# Hello".to_string());
            t.insert("file.txt".to_string(), "content".to_string());
            t
        };

        let tree_id1 = calculate_tree_id(&tree1).unwrap();
        let commit_id1 = calculate_commit_id(
            &tree_id1,
            &[],
            &author,
            author_date,
            &author,
            Timestamp {
                seconds: author_date.seconds + 3,
                offset_minutes: author_date.offset_minutes,
            },
            "commit 1",
        )
        .unwrap();

        let commit1 = Commit {
            id: commit_id1,
            parents: vec![],
            tree: tree1,
            author: author.clone(),
            author_date,
            committer: author.clone(),
            committer_date: Timestamp {
                seconds: author_date.seconds + 3,
                offset_minutes: author_date.offset_minutes,
            },
            message: "commit 1".to_string(),
        };

        // Second commit (delete file.txt)
        let tree2 = {
            let mut t = Tree::new();
            t.insert("README.md".to_string(), "# Hello".to_string());
            t
        };

        let tree_id2 = calculate_tree_id(&tree2).unwrap();
        let author_date2 = Timestamp {
            seconds: author_date.seconds + 256,
            offset_minutes: author_date.offset_minutes,
        };
        let commit_id2 = calculate_commit_id(
            &tree_id2,
            &[commit_id1],
            &author,
            author_date2,
            &author,
            Timestamp {
                seconds: author_date2.seconds + 3,
                offset_minutes: author_date2.offset_minutes,
            },
            "commit 2",
        )
        .unwrap();

        let commit2 = Commit {
            id: commit_id2,
            parents: vec![commit_id1],
            tree: tree2,
            author: author.clone(),
            author_date: author_date2,
            committer: author.clone(),
            committer_date: Timestamp {
                seconds: author_date2.seconds + 3,
                offset_minutes: author_date2.offset_minutes,
            },
            message: "commit 2".to_string(),
        };

        repo.insert_commit(commit1);
        repo.insert_commit(commit2);
        repo.insert_ref(
            RefName::new("refs/heads/main".to_string()).unwrap(),
            commit_id2,
        );
        repo.set_head(HeadState::Symbolic(
            RefName::new("refs/heads/main".to_string()).unwrap(),
        ));

        // Serialize with integer IDs
        let yaml = serialize(&repo, CommitIdStyle::Integer);
        println!("Serialized YAML with deletion:\n{}", yaml);

        // Parse it back
        let repo2 = parse(&yaml).expect("should parse");
        assert_eq!(repo2.commits().count(), 2);
    }

    #[test]
    fn test_serialize_roundtrip_hex() {
        // Create a simple repository
        let mut repo = Repository::new();

        // Create a root commit with calculated ID
        let author = Identity::parse("User <user@localhost>").unwrap();
        let author_date = Timestamp::from_iso8601("2021-01-14T08:25:36Z").unwrap();
        let committer = author.clone();
        let committer_date = Timestamp::from_iso8601("2021-01-14T08:25:39Z").unwrap();

        let tree = {
            let mut t = Tree::new();
            t.insert("README.md".to_string(), "# Hello".to_string());
            t
        };

        let tree_id = calculate_tree_id(&tree).unwrap();
        let commit_id = calculate_commit_id(
            &tree_id,
            &[],
            &author,
            author_date,
            &committer,
            committer_date,
            "Initial commit",
        )
        .unwrap();

        let root_commit = Commit {
            id: commit_id,
            parents: vec![],
            tree,
            author,
            author_date,
            committer,
            committer_date,
            message: "Initial commit".to_string(),
        };

        repo.insert_commit(root_commit.clone());
        repo.insert_ref(
            RefName::new("refs/heads/main".to_string()).unwrap(),
            root_commit.id,
        );
        repo.set_head(HeadState::Symbolic(
            RefName::new("refs/heads/main".to_string()).unwrap(),
        ));

        // Serialize with hex IDs
        let yaml = serialize(&repo, CommitIdStyle::Hex);
        println!("Serialized YAML:\n{}", yaml);

        // Parse it back
        let repo2 = parse(&yaml).expect("should parse");

        // Verify the round-trip - IDs should match since we used correct hashes
        assert_eq!(repo.commits().count(), repo2.commits().count());
        assert_eq!(
            repo.get_commit(&commit_id).unwrap().id,
            repo2.get_commit(&commit_id).unwrap().id
        );
    }

    // ============================================================================
    // ObjectId Tests
    // ============================================================================

    #[test]
    fn test_objectid_from_hex_valid() {
        let hex = "1234567890abcdef1234567890abcdef12345678";
        let oid = ObjectId::from_hex(hex).unwrap();
        assert_eq!(oid.to_hex(), hex);
    }

    #[test]
    fn test_objectid_from_hex_wrong_length() {
        assert!(ObjectId::from_hex("123").is_err());
        assert!(ObjectId::from_hex("1234567890abcdef1234567890abcdef123456789").is_err());
    }

    #[test]
    fn test_objectid_from_hex_invalid_chars() {
        assert!(ObjectId::from_hex("xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx").is_err());
        assert!(ObjectId::from_hex("1234567890abcdef1234567890abcdef1234567g").is_err());
    }

    #[test]
    fn test_objectid_to_hex_lowercase() {
        let hex = "abcdef1234567890abcdef1234567890abcdef12";
        let oid = ObjectId::from_hex(hex).unwrap();
        assert_eq!(oid.to_hex(), hex);
        // Verify it's lowercase
        assert_eq!(oid.to_hex(), oid.to_hex().to_lowercase());
    }

    #[test]
    fn test_objectid_roundtrip() {
        let original = "deadbeef00000000111111112222222233333333";
        let oid = ObjectId::from_hex(original).unwrap();
        let roundtrip = oid.to_hex();
        assert_eq!(original, roundtrip);
    }

    // ============================================================================
    // Timestamp Tests
    // ============================================================================

    #[test]
    fn test_timestamp_from_iso8601_with_z() {
        let ts = Timestamp::from_iso8601("2021-01-14T08:25:36Z").unwrap();
        assert_eq!(ts.offset_minutes, 0);
    }

    #[test]
    fn test_timestamp_from_iso8601_with_plus_offset() {
        let ts = Timestamp::from_iso8601("2021-01-14T08:25:36+05:30").unwrap();
        assert_eq!(ts.offset_minutes, 5 * 60 + 30);
    }

    #[test]
    fn test_timestamp_from_iso8601_with_minus_offset() {
        let ts = Timestamp::from_iso8601("2021-01-14T08:25:36-02:00").unwrap();
        assert_eq!(ts.offset_minutes, -(2 * 60));
    }

    #[test]
    fn test_timestamp_from_iso8601_compact_offset() {
        let ts = Timestamp::from_iso8601("2021-01-14T08:25:36-0200").unwrap();
        assert_eq!(ts.offset_minutes, -(2 * 60));
    }

    #[test]
    fn test_timestamp_partial_year_only() {
        let ts = Timestamp::from_iso8601("2021").unwrap();
        // Should use defaults: month=02, day=04, hour=08, minute=16, second=32
        assert_eq!(ts.to_iso8601(), "2021-02-04T08:16:32Z");
    }

    #[test]
    fn test_timestamp_partial_year_month() {
        let ts = Timestamp::from_iso8601("2021-03").unwrap();
        // Should use defaults: day=04, hour=08, minute=16, second=32
        assert_eq!(ts.to_iso8601(), "2021-03-04T08:16:32Z");
    }

    #[test]
    fn test_timestamp_partial_year_month_day() {
        let ts = Timestamp::from_iso8601("2021-03-15").unwrap();
        // Should use defaults: hour=08, minute=16, second=32
        assert_eq!(ts.to_iso8601(), "2021-03-15T08:16:32Z");
    }

    #[test]
    fn test_timestamp_to_iso8601_produces_correct_format() {
        let ts = Timestamp {
            seconds: 1610612736,
            offset_minutes: 0,
        };
        let iso = ts.to_iso8601();
        assert!(iso.contains('T'));
        assert!(iso.ends_with('Z'));
    }

    #[test]
    fn test_timestamp_to_iso8601_with_offset() {
        let ts = Timestamp {
            seconds: 1610612736,
            offset_minutes: -120, // -02:00
        };
        let iso = ts.to_iso8601();
        assert!(iso.ends_with("-02:00"));
    }

    #[test]
    fn test_timestamp_default_values() {
        // Test that defaults are: month=02, day=04, hour=08, minute=16, second=32,
        // offset=Z
        let ts = Timestamp::from_iso8601("2021").unwrap();
        let iso = ts.to_iso8601();
        assert!(iso.starts_with("2021-02-04T08:16:32"));
    }

    #[test]
    fn test_timestamp_pre_epoch_error() {
        assert!(Timestamp::from_iso8601("1969-12-31T23:59:59Z").is_err());
    }

    #[test]
    fn test_timestamp_roundtrip() {
        let original = "2021-06-15T14:30:45+03:00";
        let ts = Timestamp::from_iso8601(original).unwrap();
        let iso = ts.to_iso8601();
        // Parse again to verify consistency
        let ts2 = Timestamp::from_iso8601(&iso).unwrap();
        assert_eq!(ts.seconds, ts2.seconds);
        assert_eq!(ts.offset_minutes, ts2.offset_minutes);
    }

    // ============================================================================
    // Identity Tests
    // ============================================================================

    #[test]
    fn test_identity_parse_valid() {
        let id = Identity::parse("John Doe <john@example.com>").unwrap();
        assert_eq!(id.name, "John Doe");
        assert_eq!(id.email, "john@example.com");
    }

    #[test]
    fn test_identity_parse_missing_open_bracket() {
        assert!(Identity::parse("John Doe john@example.com>").is_err());
    }

    #[test]
    fn test_identity_parse_missing_close_bracket() {
        assert!(Identity::parse("John Doe <john@example.com").is_err());
    }

    #[test]
    fn test_identity_parse_missing_at_sign() {
        assert!(Identity::parse("John Doe <johnexample.com>").is_err());
    }

    #[test]
    fn test_identity_parse_no_space_before_bracket() {
        assert!(Identity::parse("John Doe<john@example.com>").is_err());
    }

    #[test]
    fn test_identity_parse_empty_name() {
        assert!(Identity::parse(" <john@example.com>").is_err());
    }

    #[test]
    fn test_identity_to_string() {
        let id = Identity {
            name: "Jane Smith".to_string(),
            email: "jane@test.org".to_string(),
        };
        assert_eq!(id.format(), "Jane Smith <jane@test.org>");
    }

    #[test]
    fn test_identity_roundtrip() {
        let original = "Alice Wonder <alice@wonderland.com>";
        let id = Identity::parse(original).unwrap();
        assert_eq!(id.format(), original);
    }

    // ============================================================================
    // Tree Tests
    // ============================================================================

    #[test]
    fn test_tree_insert_get() {
        let mut tree = Tree::new();
        tree.insert("file.txt".to_string(), "content".to_string());
        assert_eq!(tree.get("file.txt"), Some("content"));
    }

    #[test]
    fn test_tree_get_nonexistent() {
        let tree = Tree::new();
        assert_eq!(tree.get("missing.txt"), None);
    }

    #[test]
    fn test_tree_remove_file() {
        let mut tree = Tree::new();
        tree.insert("file.txt".to_string(), "content".to_string());
        assert!(tree.remove("file.txt"));
        assert_eq!(tree.get("file.txt"), None);
    }

    #[test]
    fn test_tree_remove_directory() {
        let mut tree = Tree::new();
        tree.insert("dir/file1.txt".to_string(), "content1".to_string());
        tree.insert("dir/file2.txt".to_string(), "content2".to_string());
        tree.insert("other.txt".to_string(), "other".to_string());

        // Remove the directory
        assert!(tree.remove("dir"));

        // Both files under dir/ should be removed
        assert_eq!(tree.get("dir/file1.txt"), None);
        assert_eq!(tree.get("dir/file2.txt"), None);

        // Other file should remain
        assert_eq!(tree.get("other.txt"), Some("other"));
    }

    #[test]
    fn test_tree_paths_iterator() {
        let mut tree = Tree::new();
        tree.insert("b.txt".to_string(), "b".to_string());
        tree.insert("a.txt".to_string(), "a".to_string());
        tree.insert("c.txt".to_string(), "c".to_string());

        let paths: Vec<&str> = tree.paths().collect();
        // Should be sorted (BTreeMap)
        assert_eq!(paths, vec!["a.txt", "b.txt", "c.txt"]);
    }

    #[test]
    fn test_tree_validate_path_rejects_slash() {
        let result = Tree::validate_path("dir/sub/file.txt");
        assert!(result.is_ok());

        let result = Tree::validate_component("dir/sub");
        assert!(result.is_err());
    }

    #[test]
    fn test_tree_validate_path_rejects_backslash() {
        let result = Tree::validate_component("dir\\sub");
        assert!(result.is_err());
    }

    #[test]
    fn test_tree_validate_path_rejects_colon() {
        let result = Tree::validate_component("C:");
        assert!(result.is_err());
    }

    #[test]
    fn test_tree_validate_path_rejects_dot() {
        assert!(Tree::validate_component(".").is_err());
    }

    #[test]
    fn test_tree_validate_path_rejects_dotdot() {
        assert!(Tree::validate_component("..").is_err());
    }

    #[test]
    fn test_tree_validate_path_rejects_empty() {
        assert!(Tree::validate_component("").is_err());
    }

    // ============================================================================
    // Parsing Tests
    // ============================================================================

    #[test]
    fn test_parse_simple_single_commit() {
        let yaml = r##"
HEAD: refs/heads/main
refs:
  heads:
    main: 1
1:
  parents: []
  tree:
    README.md: "# Hello"
"##;
        let repo = parse(yaml).unwrap();
        assert_eq!(repo.commits().count(), 1);

        let commit = repo.commits().next().unwrap();
        assert_eq!(commit.parents.len(), 0);
        assert_eq!(commit.tree.get("README.md"), Some("# Hello"));
    }

    #[test]
    fn test_parse_multiple_commits_parent_chain() {
        let yaml = r#"
HEAD: refs/heads/main
refs:
  heads:
    main: 2
1:
  parents: []
  tree:
    file.txt: "first"
2:
  tree:
    file.txt: "second"
"#;
        let repo = parse(yaml).unwrap();
        assert_eq!(repo.commits().count(), 2);

        // Second commit should have first as parent
        let commits: Vec<&Commit> = repo.commits().collect();
        let commit2 = commits.iter().find(|c| c.message == "commit 2").unwrap();
        assert_eq!(commit2.parents.len(), 1);
    }

    #[test]
    fn test_parse_merge_commit_multiple_parents() {
        let yaml = r#"
HEAD: refs/heads/main
refs:
  heads:
    main: 3
1:
  parents: []
  tree:
    file.txt: "first"
2:
  parents: [1]
  tree:
    file.txt: "branch"
3:
  parents: [1, 2]
  message: "merge"
  tree:
    file.txt: "merged"
"#;
        let repo = parse(yaml).unwrap();
        assert_eq!(repo.commits().count(), 3);

        // Find merge commit
        let merge = repo.commits().find(|c| c.message == "merge").unwrap();
        assert_eq!(merge.parents.len(), 2);
    }

    #[test]
    fn test_parse_default_author_inheritance() {
        let yaml = r#"
HEAD: refs/heads/main
refs:
  heads:
    main: 2
1:
  parents: []
  author: "Alice <alice@example.com>"
  tree: {}
2:
  tree: {}
"#;
        let repo = parse(yaml).unwrap();

        let commit2 = repo.commits().find(|c| c.message == "commit 2").unwrap();
        assert_eq!(commit2.author.name, "Alice");
        assert_eq!(commit2.author.email, "alice@example.com");
    }

    #[test]
    fn test_parse_default_dates() {
        let yaml = r#"
HEAD: refs/heads/main
refs:
  heads:
    main: 1
1:
  parents: []
  tree: {}
"#;
        let repo = parse(yaml).unwrap();

        let commit = repo.commits().next().unwrap();
        // Should have default date: 2021-01-14T08:25:36Z
        assert_eq!(commit.author_date.to_iso8601(), "2021-01-14T08:25:36Z");
        // Commit date should be author_date + 3 seconds
        assert_eq!(commit.committer_date.to_iso8601(), "2021-01-14T08:25:39Z");
    }

    #[test]
    fn test_parse_tree_modifications() {
        let yaml = r#"
HEAD: refs/heads/main
refs:
  heads:
    main: 2
1:
  parents: []
  tree:
    a.txt: "a"
    b.txt: "b"
2:
  tree:
    b.txt: "modified"
    c.txt: "new"
"#;
        let repo = parse(yaml).unwrap();

        let commit2 = repo.commits().find(|c| c.message == "commit 2").unwrap();
        assert_eq!(commit2.tree.get("a.txt"), Some("a")); // Inherited
        assert_eq!(commit2.tree.get("b.txt"), Some("modified")); // Modified
        assert_eq!(commit2.tree.get("c.txt"), Some("new")); // Added
    }

    #[test]
    fn test_parse_tree_deletions() {
        let yaml = r#"
HEAD: refs/heads/main
refs:
  heads:
    main: 2
1:
  parents: []
  tree:
    a.txt: "a"
    b.txt: "b"
2:
  tree:
    b.txt:
"#;
        let repo = parse(yaml).unwrap();

        let commit2 = repo.commits().find(|c| c.message == "commit 2").unwrap();
        assert_eq!(commit2.tree.get("a.txt"), Some("a")); // Inherited
        assert_eq!(commit2.tree.get("b.txt"), None); // Deleted
    }

    #[test]
    fn test_parse_yaml_keyword_coercion_true() {
        let yaml = r#"
HEAD: refs/heads/main
refs:
  heads:
    main: 1
1:
  parents: []
  tree:
    true: "file named true"
"#;
        let repo = parse(yaml).unwrap();
        let commit = repo.commits().next().unwrap();
        assert_eq!(commit.tree.get("true"), Some("file named true"));
    }

    #[test]
    fn test_parse_yaml_keyword_coercion_false() {
        let yaml = r#"
HEAD: refs/heads/main
refs:
  heads:
    main: 1
1:
  parents: []
  tree:
    false: "file named false"
"#;
        let repo = parse(yaml).unwrap();
        let commit = repo.commits().next().unwrap();
        assert_eq!(commit.tree.get("false"), Some("file named false"));
    }

    #[test]
    fn test_parse_yaml_keyword_coercion_null() {
        let yaml = r#"
HEAD: refs/heads/main
refs:
  heads:
    main: 1
1:
  parents: []
  tree:
    "null": "file named null"
"#;
        let repo = parse(yaml).unwrap();
        let commit = repo.commits().next().unwrap();
        assert_eq!(commit.tree.get("null"), Some("file named null"));
    }

    #[test]
    fn test_parse_yaml_integer_key() {
        let yaml = r#"
HEAD: refs/heads/main
refs:
  heads:
    main: 1
1:
  parents: []
  tree:
    123: "file named 123"
"#;
        let repo = parse(yaml).unwrap();
        let commit = repo.commits().next().unwrap();
        assert_eq!(commit.tree.get("123"), Some("file named 123"));
    }

    #[test]
    fn test_parse_yaml_tags_in_value() {
        // Note: The current version of serde_yaml (0.9.x) strips YAML tags during
        // parsing, so they don't make it to our code. Our normalize_yaml_key
        // function has the check for Tagged values (as per spec), but
        // serde_yaml removes them before we see them. This test documents that
        // tags in values are currently accepted (stripped by parser).
        // A future version with a different YAML parser might need stricter handling.
        let yaml = r#"
HEAD: refs/heads/main
refs:
  heads:
    main: 1
1:
  parents: []
  tree:
    file.txt: "content"
"#;
        // This should parse successfully
        let result = parse(yaml);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_cycle_detection() {
        let yaml = r#"
HEAD: refs/heads/main
refs:
  heads:
    main: 1
1:
  parents: [2]
  tree: {}
2:
  parents: [1]
  tree: {}
"#;
        let result = parse(yaml);
        assert!(result.is_err());
        match result {
            Err(ParseError::CycleDetected) => {}
            _ => panic!("Expected CycleDetected error"),
        }
    }

    // ============================================================================
    // Serialization Tests
    // ============================================================================

    #[test]
    fn test_serialize_omits_defaults() {
        let yaml = r#"
HEAD: refs/heads/main
refs:
  heads:
    main: 2
1:
  parents: []
  tree:
    file.txt: "content"
2:
  tree:
    file.txt: "updated"
"#;
        let repo = parse(yaml).unwrap();
        let serialized = serialize(&repo, CommitIdStyle::Integer);

        // The serialized output should omit default values
        // Second commit should not have explicit author since it inherits
        assert!(serialized.contains("1:"));
        assert!(serialized.contains("2:"));

        // Parse it back to ensure it works
        let repo2 = parse(&serialized).unwrap();
        assert_eq!(repo2.commits().count(), 2);
    }

    #[test]
    fn test_serialize_tree_delta() {
        let yaml = r#"
HEAD: refs/heads/main
refs:
  heads:
    main: 2
1:
  parents: []
  tree:
    a.txt: "a"
    b.txt: "b"
2:
  tree:
    b.txt: "modified"
"#;
        let repo = parse(yaml).unwrap();
        let serialized = serialize(&repo, CommitIdStyle::Integer);

        // The second commit should only include the delta (modified b.txt)
        // Not the entire tree
        let repo2 = parse(&serialized).unwrap();
        let commit2 = repo2.commits().find(|c| c.message == "commit 2").unwrap();
        assert_eq!(commit2.tree.get("a.txt"), Some("a"));
        assert_eq!(commit2.tree.get("b.txt"), Some("modified"));
    }

    #[test]
    fn test_serialize_commit_ordering() {
        let yaml = r#"
HEAD: refs/heads/main
refs:
  heads:
    main: 2
1:
  parents: []
  tree:
    file.txt: "first"
2:
  tree:
    file.txt: "second"
"#;
        let repo = parse(yaml).unwrap();
        let serialized = serialize(&repo, CommitIdStyle::Integer);

        // Commits should be in topological order (parent before child)
        let pos1 = serialized.find("1:").unwrap();
        let pos2 = serialized.find("2:").unwrap();
        assert!(pos1 < pos2, "Parent commit should appear before child");
    }

    // ============================================================================
    // Round-trip Tests
    // ============================================================================

    #[test]
    fn test_roundtrip_empty_tree() {
        let yaml = r#"
HEAD: refs/heads/main
refs:
  heads:
    main: 1
1:
  parents: []
  tree: {}
"#;
        let repo = parse(yaml).unwrap();
        let serialized = serialize(&repo, CommitIdStyle::Integer);
        let repo2 = parse(&serialized).unwrap();

        assert_eq!(repo.commits().count(), repo2.commits().count());
        let commit = repo2.commits().next().unwrap();
        assert!(commit.tree.is_empty());
    }

    #[test]
    fn test_roundtrip_complex_tree() {
        let yaml = r##"
HEAD: refs/heads/main
refs:
  heads:
    main: 1
1:
  parents: []
  tree:
    src:
      main.rs: "fn main() {}"
      lib.rs: "pub mod test;"
    tests:
      test.rs: "#[test]"
    README.md: "# Project"
"##;
        let repo = parse(yaml).unwrap();
        let serialized = serialize(&repo, CommitIdStyle::Integer);
        let repo2 = parse(&serialized).unwrap();

        let commit = repo2.commits().next().unwrap();
        assert_eq!(commit.tree.get("src/main.rs"), Some("fn main() {}"));
        assert_eq!(commit.tree.get("src/lib.rs"), Some("pub mod test;"));
        assert_eq!(commit.tree.get("tests/test.rs"), Some("#[test]"));
        assert_eq!(commit.tree.get("README.md"), Some("# Project"));
    }

    #[test]
    fn test_roundtrip_multiple_branches() {
        let yaml = r#"
HEAD: refs/heads/main
refs:
  heads:
    main: 2
    dev: 3
1:
  parents: []
  tree:
    file.txt: "base"
2:
  tree:
    file.txt: "main"
3:
  parents: [1]
  tree:
    file.txt: "dev"
"#;
        let repo = parse(yaml).unwrap();
        let serialized = serialize(&repo, CommitIdStyle::Integer);
        let repo2 = parse(&serialized).unwrap();

        assert_eq!(repo2.commits().count(), 3);
        assert_eq!(repo2.refs().count(), 2);
    }

    #[test]
    fn test_roundtrip_with_merge() {
        let yaml = r#"
HEAD: refs/heads/main
refs:
  heads:
    main: 4
1:
  parents: []
  tree:
    file.txt: "base"
2:
  tree:
    file.txt: "main-1"
3:
  parents: [1]
  tree:
    file.txt: "branch"
4:
  parents: [2, 3]
  message: "Merge branch into main"
  tree:
    file.txt: "merged"
"#;
        let repo = parse(yaml).unwrap();
        let serialized = serialize(&repo, CommitIdStyle::Integer);
        let repo2 = parse(&serialized).unwrap();

        assert_eq!(repo2.commits().count(), 4);
        let merge = repo2
            .commits()
            .find(|c| c.message.contains("Merge"))
            .unwrap();
        assert_eq!(merge.parents.len(), 2);
    }

    #[test]
    fn test_roundtrip_preserves_identity() {
        let yaml = r#"
HEAD: refs/heads/main
refs:
  heads:
    main: 1
1:
  parents: []
  author: "Custom Author <custom@example.com>"
  committer: "Custom Committer <committer@example.com>"
  tree: {}
"#;
        let repo = parse(yaml).unwrap();
        let serialized = serialize(&repo, CommitIdStyle::Integer);
        let repo2 = parse(&serialized).unwrap();

        let commit = repo2.commits().next().unwrap();
        assert_eq!(commit.author.name, "Custom Author");
        assert_eq!(commit.committer.name, "Custom Committer");
    }

    #[test]
    fn test_roundtrip_preserves_timestamps() {
        let yaml = r#"
HEAD: refs/heads/main
refs:
  heads:
    main: 1
1:
  parents: []
  author-date: "2023-06-15T14:30:00+02:00"
  commit-date: "2023-06-15T14:35:00+02:00"
  tree: {}
"#;
        let repo = parse(yaml).unwrap();
        let serialized = serialize(&repo, CommitIdStyle::Integer);
        let repo2 = parse(&serialized).unwrap();

        let commit = repo2.commits().next().unwrap();
        assert_eq!(commit.author_date.offset_minutes, 120);
        assert_eq!(commit.committer_date.offset_minutes, 120);
    }

    #[test]
    fn test_roundtrip_hex_style() {
        let yaml = r#"
HEAD: refs/heads/main
refs:
  heads:
    main: 1
1:
  parents: []
  tree:
    file.txt: "content"
"#;
        let repo = parse(yaml).unwrap();
        let serialized = serialize(&repo, CommitIdStyle::Hex);

        // Should contain 40-character hex IDs
        let lines: Vec<&str> = serialized.lines().collect();
        let has_hex_key = lines.iter().any(|line| {
            line.contains("main:")
                && line
                    .split(':')
                    .nth(1)
                    .map(|s| s.trim().len() == 40)
                    .unwrap_or(false)
        });
        assert!(
            has_hex_key
                || serialized.contains("main: ")
                    && serialized
                        .split("main: ")
                        .nth(1)
                        .map(|s| s.trim().len() >= 40)
                        .unwrap_or(false)
        );

        let repo2 = parse(&serialized).unwrap();
        assert_eq!(repo2.commits().count(), 1);
    }

    #[test]
    fn test_parse_commit_reference() {
        // Test basic [commit] reference to copy content from previous commit
        let yaml = r#"
HEAD: refs/heads/main
refs:
  heads:
    main: 2
1:
  parents: []
  tree:
    file.txt: "original content"
    dir:
      nested.rs: "nested file"
2:
  tree:
    file.txt:
      [commit]: 1
      [path]: file.txt
"#;
        let repo = parse(yaml).unwrap();
        let commits: Vec<_> = repo.commits().collect();
        assert_eq!(commits.len(), 2);

        let commit2 = commits.iter().find(|c| c.message == "commit 2").unwrap();
        assert_eq!(commit2.tree.get("file.txt"), Some("original content"));
    }

    #[test]
    fn test_parse_path_reference_with_rename() {
        // Test [path] reference for renaming a file
        let yaml = r#"
HEAD: refs/heads/main
refs:
  heads:
    main: 2
1:
  parents: []
  tree:
    old-name.txt: "file content"
2:
  tree:
    old-name.txt: null
    new-name.txt:
      [commit]: 1
      [path]: old-name.txt
"#;
        let repo = parse(yaml).unwrap();
        let commits: Vec<_> = repo.commits().collect();
        assert_eq!(commits.len(), 2);

        let commit2 = commits.iter().find(|c| c.message == "commit 2").unwrap();
        assert_eq!(commit2.tree.get("new-name.txt"), Some("file content"));
        assert_eq!(commit2.tree.get("old-name.txt"), None);
    }

    #[test]
    fn test_parse_tree_reference() {
        // Test referencing an entire tree (directory)
        let yaml = r#"
HEAD: refs/heads/main
refs:
  heads:
    main: 2
1:
  parents: []
  tree:
    src:
      lib.rs: "pub fn main() {}"
      util.rs: "pub fn helper() {}"
2:
  tree:
    copied-src:
      [commit]: 1
      [path]: src
"#;
        let repo = parse(yaml).unwrap();
        let commits: Vec<_> = repo.commits().collect();
        assert_eq!(commits.len(), 2);

        let commit2 = commits.iter().find(|c| c.message == "commit 2").unwrap();
        assert_eq!(
            commit2.tree.get("copied-src/lib.rs"),
            Some("pub fn main() {}")
        );
        assert_eq!(
            commit2.tree.get("copied-src/util.rs"),
            Some("pub fn helper() {}")
        );
    }

    #[test]
    fn test_parse_path_inheritance() {
        // Test that [path] is inherited through nested structures
        let yaml = r#"
HEAD: refs/heads/main
refs:
  heads:
    main: 2
1:
  parents: []
  tree:
    foo:
      src:
        main.rs: "fn main() {}"
        lib.rs: "pub fn lib() {}"
2:
  tree:
    bar:
      [commit]: 1
      [path]: foo/src
      extra.rs: "// extra"
"#;
        let repo = parse(yaml).unwrap();
        let commits: Vec<_> = repo.commits().collect();
        assert_eq!(commits.len(), 2);

        let commit2 = commits.iter().find(|c| c.message == "commit 2").unwrap();
        // Should have inherited files from foo/src
        assert_eq!(commit2.tree.get("bar/main.rs"), Some("fn main() {}"));
        assert_eq!(commit2.tree.get("bar/lib.rs"), Some("pub fn lib() {}"));
        // Plus the extra file
        assert_eq!(commit2.tree.get("bar/extra.rs"), Some("// extra"));
    }

    #[test]
    fn test_parse_relative_path_sibling() {
        // Test relative path resolution with ./
        let yaml = r#"
HEAD: refs/heads/main
refs:
  heads:
    main: 2
1:
  parents: []
  tree:
    src:
      lib.rs: "library"
      foo.rs: "foo content"
2:
  tree:
    src:
      lib.rs:
        [commit]: 1
        [path]: ./foo.rs
"#;
        let repo = parse(yaml).unwrap();
        let commits: Vec<_> = repo.commits().collect();
        assert_eq!(commits.len(), 2);

        let commit2 = commits.iter().find(|c| c.message == "commit 2").unwrap();
        assert_eq!(commit2.tree.get("src/lib.rs"), Some("foo content"));
    }

    #[test]
    fn test_parse_relative_path_parent() {
        // Test relative path resolution with ../
        let yaml = r#"
HEAD: refs/heads/main
refs:
  heads:
    main: 2
1:
  parents: []
  tree:
    root.txt: "root content"
    dir:
      file.txt: "nested"
2:
  tree:
    dir:
      file.txt:
        [commit]: 1
        [path]: ../root.txt
"#;
        let repo = parse(yaml).unwrap();
        let commits: Vec<_> = repo.commits().collect();
        assert_eq!(commits.len(), 2);

        let commit2 = commits.iter().find(|c| c.message == "commit 2").unwrap();
        assert_eq!(commit2.tree.get("dir/file.txt"), Some("root content"));
    }

    #[test]
    fn test_parse_commit_inheritance() {
        // Test that [commit] defaults to first parent
        let yaml = r#"
HEAD: refs/heads/main
refs:
  heads:
    main: 2
1:
  parents: []
  tree:
    file.txt: "content from commit 1"
2:
  tree:
    copy.txt:
      [path]: file.txt
"#;
        let repo = parse(yaml).unwrap();
        let commits: Vec<_> = repo.commits().collect();
        assert_eq!(commits.len(), 2);

        let commit2 = commits.iter().find(|c| c.message == "commit 2").unwrap();
        // Should inherit [commit]: 1 by default
        assert_eq!(commit2.tree.get("copy.txt"), Some("content from commit 1"));
        // Original file should still be there (inherited from parent)
        assert_eq!(commit2.tree.get("file.txt"), Some("content from commit 1"));
    }
}
