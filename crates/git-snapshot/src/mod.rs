use std::collections::{BTreeMap, HashMap};
use std::fmt;

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

    #[error("commit not found: {0}")]
    CommitNotFound(String),

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

    /// Convert to 40-character lowercase hex string
    pub fn to_hex(&self) -> String {
        self.0
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Timestamp {
    /// Seconds since Unix epoch (must be non-negative for git compatibility)
    pub seconds: i64,

    /// Timezone offset in minutes from UTC (e.g., -120 for -02:00)
    pub offset_minutes: i16,
}

impl Timestamp {
    /// Parse from ISO 8601 string with lenient parsing.
    /// Defaults: month=02, day=04, hour=08, minute=16, second=32, offset=Z (UTC/0)
    pub fn from_iso8601(s: &str) -> Result<Self, ParseError> {
        let s = s.trim();

        // Split into date and time parts (accepting various delimiters)
        let parts: Vec<&str> = s.split(|c| c == 'T' || c == 't' || c == ' ').collect();

        if parts.is_empty() {
            return Err(ParseError::InvalidTimestamp("empty string".to_string()));
        }

        // Parse date part (YYYY-MM-DD or YYYY-MM or YYYY)
        let date_part = parts[0];
        let date_components: Vec<&str> = date_part.split(|c| c == '-' || c == '/').collect();

        if date_components.is_empty() {
            return Err(ParseError::InvalidTimestamp("missing year".to_string()));
        }

        let year: i32 = date_components[0]
            .parse()
            .map_err(|_| ParseError::InvalidTimestamp(format!("invalid year: {}", date_components[0])))?;

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

            // Extract timezone offset first (can be Z, +HH:MM, -HH:MM, +HHMM, -HHMM, +HH, -HH)
            let (time_part, offset) = if time_part.ends_with('Z') || time_part.ends_with('z') {
                (&time_part[..time_part.len() - 1], 0i16)
            } else if let Some(pos) = time_part.rfind(|c| c == '+' || c == '-') {
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
                        ParseError::InvalidTimestamp(format!("invalid offset minutes: {}", parts[1]))
                    })?;
                    sign * (hours * 60 + mins)
                } else if offset_digits.len() == 4 {
                    // Format: +HHMM or -HHMM
                    let hours: i16 = offset_digits[0..2].parse().map_err(|_| {
                        ParseError::InvalidTimestamp(format!("invalid offset hours: {}", &offset_digits[0..2]))
                    })?;
                    let mins: i16 = offset_digits[2..4].parse().map_err(|_| {
                        ParseError::InvalidTimestamp(format!("invalid offset minutes: {}", &offset_digits[2..4]))
                    })?;
                    sign * (hours * 60 + mins)
                } else if offset_digits.len() == 2 {
                    // Format: +HH or -HH
                    let hours: i16 = offset_digits.parse().map_err(|_| {
                        ParseError::InvalidTimestamp(format!("invalid offset hours: {}", offset_digits))
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
        // Simplified calculation (doesn't handle all edge cases perfectly, but good enough for our use)
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
    pub fn to_iso8601(&self) -> String {
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
        let open_bracket = s.rfind('<').ok_or_else(|| {
            ParseError::InvalidIdentity("missing '<' before email".to_string())
        })?;

        let close_bracket = s.rfind('>').ok_or_else(|| {
            ParseError::InvalidIdentity("missing '>' after email".to_string())
        })?;

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
            return Err(ParseError::InvalidIdentity("name cannot be empty".to_string()));
        }

        Ok(Identity { name, email })
    }

    /// Format as git string: "Name <email@example.com>"
    pub fn to_string(&self) -> String {
        format!("{} <{}>", self.name, self.email)
    }
}

// ============================================================================
// Tree
// ============================================================================

/// The contents of a git tree (directory).
/// Stored as a flat map from full paths to blob contents.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Tree {
    /// Map from file paths to blob contents.
    /// Paths use forward slashes as separators, never have leading/trailing slashes.
    entries: BTreeMap<String, String>,
}

impl Tree {
    /// Create an empty tree
    pub fn new() -> Self {
        Tree {
            entries: BTreeMap::new(),
        }
    }

    /// Get the content of a blob at the given path
    pub fn get(&self, path: &str) -> Option<&str> {
        self.entries.get(path).map(|s| s.as_str())
    }

    /// Set the content of a blob at the given path
    pub fn insert(&mut self, path: String, content: String) {
        // Validate path components
        if let Err(_) = Self::validate_path(&path) {
            // For now, just insert anyway. Validation should be done before calling.
            // In a full implementation, we might want to return Result here.
        }
        self.entries.insert(path, content);
    }

    /// Remove a file or directory at the given path
    /// Returns true if something was removed
    pub fn remove(&mut self, path: &str) -> bool {
        // Remove the exact path if it exists
        let exact_removed = self.entries.remove(path).is_some();

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

/// The state of HEAD: either pointing to a ref (symbolic) or directly to a commit (detached).
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

pub fn main() {}
