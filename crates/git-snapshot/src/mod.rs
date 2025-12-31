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
        .get(&serde_yaml::Value::String("HEAD".to_string()))
        .ok_or(ParseError::MissingField("HEAD"))?;

    let head = parse_head(head_value)?;

    // Parse refs
    let refs_value = mapping.get(&serde_yaml::Value::String("refs".to_string()));
    let refs = if let Some(refs_value) = refs_value {
        parse_refs(refs_value)?
    } else {
        BTreeMap::new()
    };

    // Parse commits - build a map of commit references to their definitions
    let mut commit_defs = BTreeMap::new();
    let mut integer_to_hex: HashMap<u32, ObjectId> = HashMap::new();

    for (key, value) in mapping.iter() {
        let key_str = key.as_str();
        if key_str == Some("HEAD") || key_str == Some("refs") {
            continue;
        }

        // Parse commit reference
        let commit_ref = parse_commit_ref_key(key)?;

        let commit_mapping = value.as_mapping().ok_or_else(|| ParseError::UnexpectedType {
            expected: "mapping",
            actual: format!("{:?}", value),
        })?;

        commit_defs.insert(commit_ref.clone(), commit_mapping);
    }

    // First pass: compute ObjectIds for integer-keyed commits
    // We need to resolve the commit graph to compute object IDs
    // For now, we'll use a placeholder approach and compute them later

    // Build commits in document order
    let commit_order: Vec<_> = commit_defs.keys().cloned().collect();

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
            &mut commit_processing_state,
        )?;

        commits.insert(commit.id, commit);
    }

    // Convert refs to use resolved ObjectIds
    let mut resolved_refs = BTreeMap::new();
    for (ref_name, commit_ref) in refs {
        let object_id = resolve_commit_ref(&commit_ref, &integer_to_hex)?;
        resolved_refs.insert(ref_name, object_id);
    }

    // Convert HEAD to use resolved ObjectId
    let resolved_head = match head {
        HeadStateOrRef::Symbolic(ref_name) => HeadState::Symbolic(ref_name),
        HeadStateOrRef::Detached(commit_ref) => {
            let object_id = resolve_commit_ref(&commit_ref, &integer_to_hex)?;
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
    Int(u32),
}

#[derive(Debug, Clone)]
enum CommitProcessingState {
    InProgress,
    Complete(Commit),
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
                return Err(ParseError::UnexpectedType {
                    expected: "ref name or commit ID",
                    actual: s.to_string(),
                });
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
    let mapping = value.as_mapping().ok_or_else(|| ParseError::UnexpectedType {
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
        if s.len() == 40 {
            let oid = ObjectId::from_hex(s)?;
            Ok(CommitRef::Hex(oid))
        } else {
            Err(ParseError::InvalidObjectId(format!(
                "commit key must be 40-char hex or positive integer, got: {}",
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
        if s.len() == 40 {
            let oid = ObjectId::from_hex(s)?;
            Ok(CommitRef::Hex(oid))
        } else {
            Err(ParseError::InvalidObjectId(format!(
                "commit reference must be 40-char hex or positive integer, got: {}",
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
) -> Result<ObjectId, ParseError> {
    match commit_ref {
        CommitRef::Hex(oid) => Ok(*oid),
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

fn build_commit(
    commit_ref: &CommitRef,
    commit_defs: &BTreeMap<CommitRef, &serde_yaml::Mapping>,
    commit_order: &[CommitRef],
    _idx: usize,
    prev_commit_ref: Option<&CommitRef>,
    integer_to_hex: &mut HashMap<u32, ObjectId>,
    processing_state: &mut HashMap<CommitRef, CommitProcessingState>,
) -> Result<Commit, ParseError> {
    // Check for cycles
    if let Some(CommitProcessingState::InProgress) = processing_state.get(commit_ref) {
        return Err(ParseError::CycleDetected);
    }

    // Check if already processed
    if let Some(CommitProcessingState::Complete(commit)) = processing_state.get(commit_ref) {
        return Ok(commit.clone());
    }

    processing_state.insert(commit_ref.clone(), CommitProcessingState::InProgress);

    let commit_mapping = commit_defs.get(commit_ref).ok_or_else(|| {
        ParseError::CommitNotFound(format!("commit {:?} not found", commit_ref))
    })?;

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
                    ParseError::CommitNotFound(format!("parent commit {:?} not in order", parent_ref))
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
    let tree = parse_tree(commit_mapping, first_parent)?;

    // Calculate object ID
    let parent_ids: Vec<ObjectId> = resolved_parents.iter().map(|c| c.id).collect();
    let tree_id = calculate_tree_id(&tree)?;
    let object_id = calculate_commit_id(
        &tree_id,
        &parent_ids,
        &author,
        author_date,
        &committer,
        committer_date,
        &message,
    )?;

    // If this is an integer reference, store the mapping
    if let CommitRef::Int(n) = commit_ref {
        integer_to_hex.insert(*n, object_id);
    }

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

    processing_state.insert(commit_ref.clone(), CommitProcessingState::Complete(commit.clone()));

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

        let parents_seq = parents_value.as_sequence().ok_or_else(|| ParseError::UnexpectedType {
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
        let author_str = author_value.as_str().ok_or_else(|| ParseError::UnexpectedType {
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
        // Default: 256 seconds after max parent author-date, or 2021-01-14T08:25:36Z for first commit
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
            CommitRef::Hex(_) => Ok(format!("commit at {}", committer_date.to_iso8601())),
        }
    }
}

fn parse_tree(
    mapping: &serde_yaml::Mapping,
    first_parent: Option<&Commit>,
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
                apply_tree_delta(&mut tree, "", tree_mapping)?;
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

fn apply_tree_delta(
    tree: &mut Tree,
    prefix: &str,
    mapping: &serde_yaml::Mapping,
) -> Result<(), ParseError> {
    for (key, value) in mapping.iter() {
        let name = normalize_yaml_key(key)?;

        // Validate the name component
        Tree::validate_component(&name)?;

        let path = if prefix.is_empty() {
            name.clone()
        } else {
            format!("{}/{}", prefix, name)
        };

        if value.is_null() {
            // Delete
            tree.remove(&path);
        } else if let Some(s) = value.as_str() {
            // Blob content
            tree.insert(path, s.to_string());
        } else if let Some(nested_mapping) = value.as_mapping() {
            if nested_mapping.is_empty() {
                // Empty mapping means delete
                tree.remove(&path);
            } else {
                // Recursively apply nested modifications
                apply_tree_delta(tree, &path, nested_mapping)?;
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
            .or_insert_with(BTreeMap::new)
            .insert(name.to_string(), ("100644".to_string(), blob_oid));
    }

    // Build tree objects bottom-up
    // Start from deepest directories and work up
    let mut tree_oids: HashMap<String, ObjectId> = HashMap::new();

    // Sort directories by depth (deepest first)
    let mut dirs: Vec<String> = dir_entries.keys().cloned().collect();
    dirs.sort_by(|a, b| {
        let a_depth = if a.is_empty() { 0 } else { a.matches('/').count() + 1 };
        let b_depth = if b.is_empty() { 0 } else { b.matches('/').count() + 1 };
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
    Ok(*tree_oids.get("").unwrap())
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
        author.to_string(),
        author_date.seconds,
        format_git_offset(author_date.offset_minutes)
    ));

    commit_content.push_str(&format!(
        "committer {} {} {:+05}\n",
        committer.to_string(),
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
            if let Some(id) = repo.refs.get(ref_name) {
                if repo.commits.contains_key(id) {
                    to_visit.push(*id);
                }
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

pub fn main() {}
