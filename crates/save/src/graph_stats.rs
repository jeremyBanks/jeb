//! Graph statistics calculation with z-mode support for depth-limited scanning.
//!
//! This module implements the core algorithm for calculating commit statistics:
//! - `revision_index`: count along first-parent chain
//! - `generation_index`: maximum topological distance from roots
//! - `commit_index`: total number of reachable commits
//! - origin: identifier derived from root commit(s)
//!
//! The algorithm supports depth-limited scanning ("z-mode") to bound complexity
//! in large repositories.
use std::{
    collections::{
        HashMap,
        HashSet,
    },
    fmt::Debug,
    hash::Hash,
};
/// Statistics about a commit's position in the repository graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GraphStats {
    pub revision_index: u32,
    pub generation_index: u32,
    pub commit_index: u32,
    /// Origin: last 4 hex digits of root commit ID(s)
    /// None for root commits (r0/s0/z0)
    pub origin: Option<u16>,
    /// Whether this calculation used z-mode (hit depth limit)
    pub z_mode: bool,
}
/// Parsed commit message in our format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParsedMessage {
    pub prefix: MessagePrefix,
    pub revision_index: u32,
    pub generation_index: Option<u32>,
    pub commit_index: Option<u32>,
    pub origin: Option<u16>,
}
/// Commit message prefix indicating repository state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessagePrefix {
    Regular,
    Shallow,
    ZMode,
}
/// Abstract interface for commit data needed by the graph statistics algorithm.
pub trait CommitView: Clone + Debug {
    type Id: Clone + Eq + Hash + Ord + Debug;
    /// Get this commit's unique identifier.
    fn id(&self) -> Self::Id;
    /// Get the IDs of this commit's parent commits.
    fn parent_ids(&self) -> Vec<Self::Id>;
    /// Get the commit message (first line only).
    fn summary(&self) -> Option<String>;
    /// Get this commit's tree identifier (for validating parsed messages).
    fn tree_id(&self) -> Self::Id;
    /// Get the raw bytes of this commit's ID (for origin calculation).
    /// For SHA1, this should be 20 bytes.
    fn id_bytes(&self) -> Vec<u8>;
}
/// Abstract interface for repository operations needed by the algorithm.
pub trait RepositoryView<'repo> {
    type Commit: CommitView + 'repo;
    /// Check if this is a shallow clone.
    fn is_shallow(&self) -> bool;
    /// Find a commit by its ID.
    fn find_commit(&'repo self, id: <Self::Commit as CommitView>::Id) -> Option<Self::Commit>;
    /// Validate that a tree prefix matches the actual tree ID.
    /// This is used to verify parsed commit messages.
    fn validate_tree_prefix(
        &self,
        tree_id: &<Self::Commit as CommitView>::Id,
        prefix: &str,
    ) -> bool;
}
/// Parser for commit messages in our format.
#[derive(Debug, Clone, Copy)]
pub struct MessageParser;
impl MessageParser {
    /// Parse a commit message to extract graph statistics.
    ///
    /// Returns None if the message doesn't match our format or is untrusted.
    #[must_use]
    pub fn parse(message: &str) -> Option<ParsedMessage> {
        let parts: Vec<&str> = message.split(" / ").collect();
        if parts.is_empty() {
            return None;
        }
        let first_part = parts[0].trim();
        let (prefix, revision_str) = if let Some(rest) = first_part.strip_prefix('r') {
            (MessagePrefix::Regular, rest)
        } else if let Some(rest) = first_part.strip_prefix('s') {
            (MessagePrefix::Shallow, rest)
        } else if let Some(rest) = first_part.strip_prefix('z') {
            (MessagePrefix::ZMode, rest)
        } else {
            return None;
        };
        let revision_index: u32 = revision_str.parse().ok()?;
        let mut generation_index = None;
        let mut commit_index = None;
        let mut origin = None;
        for part in &parts[1..] {
            let part = part.trim();
            if let Some(g) = part.strip_prefix('g') {
                generation_index = Some(g.parse().ok()?);
            } else if let Some(n) = part.strip_prefix('n') {
                commit_index = Some(n.parse().ok()?);
            } else if let Some(o) = part.strip_prefix('o') {
                origin = Some(u16::from_str_radix(o, 16).ok()?);
            }
        }
        Some(ParsedMessage {
            prefix,
            revision_index,
            generation_index,
            commit_index,
            origin,
        })
    }

    /// Validate a parsed message against actual commit data.
    ///
    /// Returns true if the message can be trusted based on:
    /// - Tree hash matches (if present in message)
    /// - Prefix matches repository state (s only in shallow repos)
    pub fn validate<'repo, R: RepositoryView<'repo>>(
        repo: &R,
        commit: &R::Commit,
        message: &str,
        parsed: &ParsedMessage,
    ) -> bool {
        let parts: Vec<&str> = message.split(" / ").collect();
        for part in &parts[1..] {
            let part = part.trim();
            if let Some(tree_prefix) = part.strip_prefix('x') {
                if !repo.validate_tree_prefix(&commit.tree_id(), tree_prefix) {
                    return false;
                }
            }
        }
        if parsed.prefix == MessagePrefix::Shallow && !repo.is_shallow() {
            return false;
        }
        true
    }
}
/// Calculator for graph statistics with z-mode support.
pub struct GraphStatsCalculator<'repo, 'a: 'repo, R: RepositoryView<'repo>> {
    repo: &'a R,
    max_depth: i32,
    trust_messages: bool,
    _phantom: std::marker::PhantomData<&'repo ()>,
}
impl<'repo, 'a: 'repo, R: RepositoryView<'repo>> Debug for GraphStatsCalculator<'repo, 'a, R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GraphStatsCalculator")
            .field("max_depth", &self.max_depth)
            .field("trust_messages", &self.trust_messages)
            .finish_non_exhaustive()
    }
}
impl<'repo, 'a: 'repo, R: RepositoryView<'repo>> GraphStatsCalculator<'repo, 'a, R> {
    pub const fn new(repo: &'a R, max_depth: i32) -> Self {
        Self {
            repo,
            max_depth,
            trust_messages: true,
            _phantom: std::marker::PhantomData,
        }
    }

    pub const fn new_rebuild(repo: &'a R, max_depth: i32) -> Self {
        Self {
            repo,
            max_depth,
            trust_messages: false,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Calculate graph statistics for a commit.
    ///
    /// This implements the z-mode algorithm:
    /// 1. Scan all parent paths up to `max_depth`
    /// 2. Trust r commits (if not shallow) or s commits (if shallow)
    /// 3. Don't trust z commits during initial scan
    /// 4. If ANY path hits depth limit: enter z-mode
    /// 5. Retrospectively trust collected z commits
    /// 6. For paths with no valid commit: declare z0
    pub fn calculate(&self, head: &R::Commit) -> GraphStats {
        let is_shallow = self.repo.is_shallow();
        let unlimited_depth = self.max_depth < 0;
        if self.max_depth == 0 {
            return GraphStats {
                revision_index: 0,
                generation_index: 0,
                commit_index: 0,
                origin: None,
                z_mode: true,
            };
        }
        if unlimited_depth && self.trust_messages {
            if let Some(summary) = head.summary() {
                if let Some(parsed) = MessageParser::parse(&summary) {
                    if MessageParser::validate(self.repo, head, &summary, &parsed) {
                        let trusted = match parsed.prefix {
                            MessagePrefix::Regular if !is_shallow => true,
                            MessagePrefix::Shallow if is_shallow => true,
                            _ => false,
                        };
                        if trusted {
                            let generation_index =
                                parsed.generation_index.unwrap_or(parsed.revision_index) + 1;
                            let commit_index = parsed.commit_index.unwrap_or_else(|| {
                                parsed.generation_index.unwrap_or(parsed.revision_index)
                            }) + 1;
                            return GraphStats {
                                revision_index: parsed.revision_index + 1,
                                generation_index,
                                commit_index,
                                origin: parsed.origin,
                                z_mode: false,
                            };
                        }
                    }
                }
            }
        }
        if unlimited_depth {
            self.full_graph_walk(head, is_shallow, unlimited_depth)
        } else {
            self.depth_limited_scan(head, is_shallow)
        }
    }

    /// Depth-limited scan implementing z-mode algorithm.
    fn depth_limited_scan(&self, head: &R::Commit, is_shallow: bool) -> GraphStats {
        let max_depth = self.max_depth as usize;
        let mut visited = HashSet::new();
        let mut commit_map: HashMap<_, R::Commit> = HashMap::new();
        let mut parent_map: HashMap<_, Vec<_>> = HashMap::new();
        let mut z_commits = HashSet::new();
        let mut boundary_commits = HashSet::new();
        // Store parsed stats from trusted boundary commits
        let mut trusted_stats: HashMap<_, ParsedMessage> = HashMap::new();
        let mut queue: Vec<(_, usize)> = vec![(head.id(), 0)];
        visited.insert(head.id());
        commit_map.insert(head.id(), head.clone());
        let mut hit_depth_limit = false;
        while let Some((id, depth)) = queue.pop() {
            if let Some(commit) = commit_map.get(&id).cloned() {
                let parents = commit.parent_ids();
                let should_continue = if depth == 0 {
                    true
                } else {
                    let trusted_parsed = if self.trust_messages {
                        if let Some(summary) = commit.summary() {
                            if let Some(parsed) = MessageParser::parse(&summary) {
                                if MessageParser::validate(self.repo, &commit, &summary, &parsed) {
                                    if parsed.prefix == MessagePrefix::ZMode {
                                        z_commits.insert(id.clone());
                                        None
                                    } else {
                                        match parsed.prefix {
                                            MessagePrefix::Regular if !is_shallow => Some(parsed),
                                            MessagePrefix::Shallow if is_shallow => Some(parsed),
                                            _ => None,
                                        }
                                    }
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    if let Some(parsed) = trusted_parsed {
                        trusted_stats.insert(id.clone(), parsed);
                        boundary_commits.insert(id.clone());
                        false
                    } else if depth >= max_depth {
                        hit_depth_limit = true;
                        boundary_commits.insert(id.clone());
                        false
                    } else {
                        true
                    }
                };
                parent_map.insert(id.clone(), parents.clone());
                if should_continue {
                    for parent_id in parents {
                        if visited.insert(parent_id.clone()) {
                            if let Some(parent) = self.repo.find_commit(parent_id.clone()) {
                                commit_map.insert(parent_id.clone(), parent);
                                queue.push((parent_id, depth + 1));
                            } else {
                                // Parent doesn't exist (shallow clone boundary)
                                boundary_commits.insert(id.clone());
                            }
                        } else {
                            // Parent was already visited - check if it exists
                            // This handles the case where multiple commits share a non-existent parent
                            if !commit_map.contains_key(&parent_id) {
                                boundary_commits.insert(id.clone());
                            }
                        }
                    }
                } else if parents.is_empty() {
                    boundary_commits.insert(id.clone());
                }
            }
        }
        let z_mode = hit_depth_limit;
        if z_mode {
            // In z-mode, also trust z commits we collected
            let z_commits_vec: Vec<_> = z_commits.iter().cloned().collect();
            for z_id in z_commits_vec {
                if let Some(commit) = commit_map.get(&z_id) {
                    if let Some(summary) = commit.summary() {
                        if let Some(parsed) = MessageParser::parse(&summary) {
                            if MessageParser::validate(self.repo, commit, &summary, &parsed)
                                && parsed.prefix == MessagePrefix::ZMode
                            {
                                trusted_stats.insert(z_id.clone(), parsed);
                            }
                        }
                    }
                }
            }
        }
        // Calculate revision_index: count along first-parent chain, adding trusted stats
        let revision_index = {
            let mut count = 0u32;
            let mut current_id = head.id();
            loop {
                // If we hit a trusted boundary, add its revision_index and stop
                // count = number of hops from head to here
                // boundary's revision_index = position of boundary in chain
                // so total = boundary position + hops from boundary to head
                if let Some(parsed) = trusted_stats.get(&current_id) {
                    count += parsed.revision_index;
                    break;
                }
                // If we hit an untrusted boundary (e.g., depth limit), stop
                if boundary_commits.contains(&current_id) {
                    break;
                }
                if let Some(parents) = parent_map.get(&current_id) {
                    if parents.is_empty() {
                        break;
                    }
                    let next_parent = parents[0].clone();
                    // Check if the parent exists in our explored graph before counting
                    // This handles shallow clone boundaries where the parent commit doesn't exist
                    if !parent_map.contains_key(&next_parent) && !boundary_commits.contains(&next_parent) {
                        break;
                    }
                    count += 1;
                    current_id = next_parent;
                } else {
                    break;
                }
            }
            count
        };
        let generation_index =
            self.calculate_generation_bounded(&parent_map, &head.id(), &boundary_commits, &trusted_stats);
        let commit_index = self.calculate_commit_index_bounded(&visited, &trusted_stats);
        // Calculate origin: prefer origin from trusted commits on first-parent chain
        let origin = if revision_index == 0 {
            None
        } else {
            self.calculate_origin_bounded(&parent_map, &commit_map, &boundary_commits, &trusted_stats)
        };
        GraphStats {
            revision_index,
            generation_index,
            commit_index,
            origin,
            z_mode,
        }
    }

    /// Calculate generation index for bounded graph (depth-limited scan).
    fn calculate_generation_bounded(
        &self,
        parent_map: &HashMap<<R::Commit as CommitView>::Id, Vec<<R::Commit as CommitView>::Id>>,
        head_id: &<R::Commit as CommitView>::Id,
        boundary_commits: &HashSet<<R::Commit as CommitView>::Id>,
        trusted_stats: &HashMap<<R::Commit as CommitView>::Id, ParsedMessage>,
    ) -> u32 {
        let mut distances = HashMap::new();
        // Initialize distances for boundary commits
        // For trusted boundaries, use their generation_index as the base
        // For untrusted boundaries (depth limit), use 0
        for boundary_id in boundary_commits {
            let base_gen = if let Some(parsed) = trusted_stats.get(boundary_id) {
                parsed.generation_index.unwrap_or(parsed.revision_index)
            } else {
                0
            };
            distances.insert(boundary_id.clone(), base_gen);
        }
        let mut processed = HashSet::new();
        let mut max_distance = 0;
        fn visit<Id: Clone + Eq + Hash + Ord + Debug>(
            id: &Id,
            parent_map: &HashMap<Id, Vec<Id>>,
            distances: &mut HashMap<Id, u32>,
            processed: &mut HashSet<Id>,
            max_distance: &mut u32,
        ) -> u32 {
            if let Some(&dist) = distances.get(id) {
                return dist;
            }
            if !processed.insert(id.clone()) {
                return 0;
            }
            let parents = parent_map.get(id).map(Vec::as_slice).unwrap_or(&[]);
            let max_parent_dist = parents
                .iter()
                .map(|p| {
                    visit(
                        p,
                        parent_map,
                        distances,
                        processed,
                        max_distance,
                    )
                })
                .max()
                .unwrap_or(0);
            let dist = max_parent_dist + 1;
            distances.insert(id.clone(), dist);
            if dist > *max_distance {
                *max_distance = dist;
            }
            dist
        }
        visit(
            head_id,
            parent_map,
            &mut distances,
            &mut processed,
            &mut max_distance,
        );
        max_distance
    }

    /// Calculate commit index for bounded graph, adding trusted stats.
    fn calculate_commit_index_bounded(
        &self,
        visited: &HashSet<<R::Commit as CommitView>::Id>,
        trusted_stats: &HashMap<<R::Commit as CommitView>::Id, ParsedMessage>,
    ) -> u32 {
        // Start with the commits we actually visited (minus 1 for the current commit)
        let mut count = visited.len().saturating_sub(1) as u32;
        // Add commit_index from all trusted boundaries
        for parsed in trusted_stats.values() {
            let trusted_commit_index = parsed.commit_index.unwrap_or_else(|| {
                parsed.generation_index.unwrap_or(parsed.revision_index)
            });
            count += trusted_commit_index;
        }
        count
    }

    /// Calculate origin for bounded graph.
    ///
    /// If all boundaries have trusted origins that match, use that origin.
    /// Otherwise, fall back to calculating from the boundary commit IDs.
    fn calculate_origin_bounded(
        &self,
        _parent_map: &HashMap<<R::Commit as CommitView>::Id, Vec<<R::Commit as CommitView>::Id>>,
        commit_map: &HashMap<<R::Commit as CommitView>::Id, R::Commit>,
        boundary_commits: &HashSet<<R::Commit as CommitView>::Id>,
        trusted_stats: &HashMap<<R::Commit as CommitView>::Id, ParsedMessage>,
    ) -> Option<u16> {
        // Collect trusted origins from boundary commits
        let trusted_origins: Vec<u16> = boundary_commits
            .iter()
            .filter_map(|id| trusted_stats.get(id))
            .filter_map(|parsed| parsed.origin)
            .collect();

        // If we have trusted origins, check if they all agree
        if !trusted_origins.is_empty() {
            let first = trusted_origins[0];
            if trusted_origins.iter().all(|&o| o == first) {
                return Some(first);
            }
            // If trusted origins disagree, hash them together
            use sha1::{
                Digest,
                Sha1,
            };
            let mut hasher = Sha1::new();
            let mut sorted_origins: Vec<u16> = trusted_origins.clone();
            sorted_origins.sort();
            for origin in sorted_origins {
                hasher.update(origin.to_be_bytes());
            }
            let hash = hasher.finalize();
            return Some(u16::from_be_bytes([hash[18], hash[19]]));
        }

        // Fall back to calculating from boundary commit IDs (for untrusted boundaries)
        let mut roots: Vec<_> = boundary_commits
            .iter()
            .filter_map(|id| commit_map.get(id))
            .collect();
        if roots.is_empty() {
            return None;
        }
        roots.sort_by_key(|c| c.id());
        if roots.len() == 1 {
            let bytes = roots[0].id_bytes();
            if bytes.len() >= 2 {
                let last_two = &bytes[bytes.len() - 2..];
                Some(u16::from_be_bytes([last_two[0], last_two[1]]))
            } else {
                Some(0x0000)
            }
        } else {
            use sha1::{
                Digest,
                Sha1,
            };
            let mut hasher = Sha1::new();
            for root in &roots {
                hasher.update(root.id_bytes());
            }
            let hash = hasher.finalize();
            Some(u16::from_be_bytes([hash[18], hash[19]]))
        }
    }

    fn full_graph_walk(
        &self,
        head: &R::Commit,
        _is_shallow: bool,
        _unlimited_depth: bool,
    ) -> GraphStats {
        let mut visited = HashSet::new();
        let mut queue = vec![head.id()];
        let mut commit_map: HashMap<_, R::Commit> = HashMap::new();
        let mut parent_map: HashMap<_, Vec<_>> = HashMap::new();
        visited.insert(head.id());
        commit_map.insert(head.id(), head.clone());
        while let Some(id) = queue.pop() {
            if let Some(commit) = commit_map.get(&id) {
                let parents = commit.parent_ids();
                parent_map.insert(id.clone(), parents.clone());
                for parent_id in parents {
                    if visited.insert(parent_id.clone()) {
                        if let Some(parent) = self.repo.find_commit(parent_id.clone()) {
                            commit_map.insert(parent_id.clone(), parent);
                            queue.push(parent_id);
                        }
                    }
                }
            }
        }
        let revision_index = {
            let mut count = 0;
            let mut current_id = head.id();
            while let Some(parents) = parent_map.get(&current_id) {
                if parents.is_empty() {
                    break;
                }
                count += 1;
                current_id = parents[0].clone();
            }
            count
        };
        let generation_index = self.calculate_generation(&parent_map, &head.id());
        let commit_index = visited.len().saturating_sub(1) as u32;
        let origin = if revision_index == 0 {
            None
        } else {
            self.calculate_origin(&parent_map, &commit_map)
        };
        GraphStats {
            revision_index,
            generation_index,
            commit_index,
            origin,
            z_mode: false,
        }
    }

    fn calculate_generation(
        &self,
        parent_map: &HashMap<<R::Commit as CommitView>::Id, Vec<<R::Commit as CommitView>::Id>>,
        head_id: &<R::Commit as CommitView>::Id,
    ) -> u32 {
        let mut distances = HashMap::new();
        for (id, parents) in parent_map {
            if parents.is_empty() {
                distances.insert(id.clone(), 0);
            }
        }
        let mut processed = HashSet::new();
        let mut max_distance = 0;
        fn visit<Id: Clone + Eq + Hash + Ord + Debug>(
            id: &Id,
            parent_map: &HashMap<Id, Vec<Id>>,
            distances: &mut HashMap<Id, u32>,
            processed: &mut HashSet<Id>,
            max_distance: &mut u32,
        ) -> u32 {
            if let Some(&dist) = distances.get(id) {
                return dist;
            }
            if !processed.insert(id.clone()) {
                return 0;
            }
            let parents = parent_map.get(id).map(Vec::as_slice).unwrap_or(&[]);
            let max_parent_dist = parents
                .iter()
                .map(|p| visit(p, parent_map, distances, processed, max_distance))
                .max()
                .unwrap_or(0);
            let dist = max_parent_dist + 1;
            distances.insert(id.clone(), dist);
            if dist > *max_distance {
                *max_distance = dist;
            }
            dist
        }
        visit(
            head_id,
            parent_map,
            &mut distances,
            &mut processed,
            &mut max_distance,
        );
        max_distance
    }

    fn calculate_origin(
        &self,
        parent_map: &HashMap<<R::Commit as CommitView>::Id, Vec<<R::Commit as CommitView>::Id>>,
        commit_map: &HashMap<<R::Commit as CommitView>::Id, R::Commit>,
    ) -> Option<u16> {
        let mut roots: Vec<_> = parent_map
            .iter()
            .filter(|(_id, parents)| {
                if parents.is_empty() {
                    true
                } else {
                    parents.iter().all(|p| !commit_map.contains_key(p))
                }
            })
            .filter_map(|(id, _)| commit_map.get(id))
            .collect();
        if roots.is_empty() {
            return None;
        }
        roots.sort_by_key(|c| c.id());
        if roots.len() == 1 {
            let bytes = roots[0].id_bytes();
            if bytes.len() >= 2 {
                let last_two = &bytes[bytes.len() - 2..];
                Some(u16::from_be_bytes([last_two[0], last_two[1]]))
            } else {
                Some(0x0000)
            }
        } else {
            use sha1::{
                Digest,
                Sha1,
            };
            let mut hasher = Sha1::new();
            for root in &roots {
                hasher.update(root.id_bytes());
            }
            let hash = hasher.finalize();
            Some(u16::from_be_bytes([hash[18], hash[19]]))
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    struct MockId(String);
    #[derive(Debug, Clone)]
    struct MockCommit {
        id: MockId,
        parent_ids: Vec<MockId>,
        message: Option<String>,
        tree_id: MockId,
    }
    impl CommitView for MockCommit {
        type Id = MockId;

        fn id(&self) -> Self::Id {
            self.id.clone()
        }

        fn parent_ids(&self) -> Vec<Self::Id> {
            self.parent_ids.clone()
        }

        fn summary(&self) -> Option<String> {
            self.message.clone()
        }

        fn tree_id(&self) -> Self::Id {
            self.tree_id.clone()
        }

        fn id_bytes(&self) -> Vec<u8> {
            self.id.0.as_bytes().to_vec()
        }
    }
    struct MockRepo {
        commits: HashMap<MockId, MockCommit>,
        is_shallow: bool,
    }
    impl MockRepo {
        fn new(is_shallow: bool) -> Self {
            Self {
                commits: HashMap::new(),
                is_shallow,
            }
        }

        fn add_commit(&mut self, id: &str, parents: Vec<&str>, message: Option<&str>) -> MockId {
            let mock_id = MockId(id.to_string());
            let commit = MockCommit {
                id: mock_id.clone(),
                parent_ids: parents.iter().map(|p| MockId(p.to_string())).collect(),
                message: message.map(String::from),
                tree_id: MockId(format!("tree-{}", id)),
            };
            self.commits.insert(mock_id.clone(), commit);
            mock_id
        }
    }
    impl<'repo> RepositoryView<'repo> for MockRepo {
        type Commit = MockCommit;

        fn is_shallow(&self) -> bool {
            self.is_shallow
        }

        fn find_commit(&'repo self, id: MockId) -> Option<Self::Commit> {
            self.commits.get(&id).cloned()
        }

        fn validate_tree_prefix(&self, _tree_id: &MockId, _prefix: &str) -> bool {
            true
        }
    }
    #[test]
    fn test_parse_r0() {
        let parsed = MessageParser::parse("r0 / x1234").unwrap();
        assert_eq!(parsed.prefix, MessagePrefix::Regular);
        assert_eq!(parsed.revision_index, 0);
        assert_eq!(parsed.origin, None);
    }
    #[test]
    fn test_parse_r5_with_origin() {
        let parsed = MessageParser::parse("r5 / xABCD / o1234").unwrap();
        assert_eq!(parsed.prefix, MessagePrefix::Regular);
        assert_eq!(parsed.revision_index, 5);
        assert_eq!(parsed.origin, Some(0x1234));
    }
    #[test]
    fn test_parse_s10_shallow() {
        let parsed = MessageParser::parse("s10 / g15 / n71 / xABCD / o5678").unwrap();
        assert_eq!(parsed.prefix, MessagePrefix::Shallow);
        assert_eq!(parsed.revision_index, 10);
        assert_eq!(parsed.generation_index, Some(15));
        assert_eq!(parsed.commit_index, Some(71));
        assert_eq!(parsed.origin, Some(0x5678));
    }
    #[test]
    fn test_parse_z0() {
        let parsed = MessageParser::parse("z0 / x0000").unwrap();
        assert_eq!(parsed.prefix, MessagePrefix::ZMode);
        assert_eq!(parsed.revision_index, 0);
        assert_eq!(parsed.origin, None);
    }
    #[test]
    fn test_parse_invalid() {
        assert!(MessageParser::parse("invalid").is_none());
        assert!(MessageParser::parse("x123").is_none());
        assert!(MessageParser::parse("").is_none());
    }
    #[test]
    fn test_linear_chain_no_messages() {
        let mut repo = MockRepo::new(false);
        repo.add_commit("c0", vec![], None);
        repo.add_commit("c1", vec!["c0"], None);
        repo.add_commit("c2", vec!["c1"], None);
        let head_id = repo.add_commit("c3", vec!["c2"], None);
        let head = repo.commits.get(&head_id).unwrap();
        let calculator = GraphStatsCalculator::new(&repo, -1);
        let stats = calculator.calculate(head);
        assert_eq!(stats.revision_index, 3);
        assert_eq!(stats.generation_index, 3);
        assert_eq!(stats.commit_index, 3);
        assert!(!stats.z_mode);
        assert!(stats.origin.is_some());
    }
    #[test]
    fn test_linear_chain_with_trusted_messages() {
        let mut repo = MockRepo::new(false);
        repo.add_commit("c0", vec![], None);
        repo.add_commit("c1", vec!["c0"], Some("r0 / x0000"));
        let head_id = repo.add_commit("c2", vec!["c1"], Some("r1 / x1111 / o1234"));
        let head = repo.commits.get(&head_id).unwrap();
        let calculator = GraphStatsCalculator::new(&repo, -1);
        let stats = calculator.calculate(head);
        assert_eq!(stats.revision_index, 2);
        assert!(!stats.z_mode);
        assert_eq!(stats.origin, Some(0x1234));
    }
    #[test]
    fn test_depth_limit_triggers_z_mode() {
        let mut repo = MockRepo::new(false);
        repo.add_commit("c0", vec![], None);
        repo.add_commit("c1", vec!["c0"], None);
        repo.add_commit("c2", vec!["c1"], None);
        repo.add_commit("c3", vec!["c2"], None);
        repo.add_commit("c4", vec!["c3"], None);
        let head_id = repo.add_commit("c5", vec!["c4"], None);
        let head = repo.commits.get(&head_id).unwrap();
        let calculator = GraphStatsCalculator::new(&repo, 2);
        let stats = calculator.calculate(head);
        assert!(stats.z_mode);
        assert_eq!(stats.revision_index, 2);
    }
    #[test]
    fn test_depth_limit_with_trusted_commit() {
        let mut repo = MockRepo::new(false);
        repo.add_commit("c0", vec![], None);
        repo.add_commit("c1", vec!["c0"], Some("r0 / x0000"));
        repo.add_commit("c2", vec!["c1"], None);
        let head_id = repo.add_commit("c3", vec!["c2"], None);
        let head = repo.commits.get(&head_id).unwrap();
        let calculator = GraphStatsCalculator::new(&repo, 5);
        let stats = calculator.calculate(head);
        assert!(!stats.z_mode);
        assert_eq!(stats.revision_index, 2);
    }
    #[test]
    fn test_shallow_repo_boundary() {
        let mut repo = MockRepo::new(true);
        let head_id = repo.add_commit("c1", vec!["c0"], Some("s0 / x0000"));
        let head = repo.commits.get(&head_id).unwrap();
        let calculator = GraphStatsCalculator::new(&repo, -1);
        let stats = calculator.calculate(head);
        assert_eq!(stats.revision_index, 1);
        assert!(!stats.z_mode);
    }
    #[test]
    fn test_max_depth_zero() {
        let mut repo = MockRepo::new(false);
        repo.add_commit("c0", vec![], None);
        let head_id = repo.add_commit("c1", vec!["c0"], None);
        let head = repo.commits.get(&head_id).unwrap();
        let calculator = GraphStatsCalculator::new(&repo, 0);
        let stats = calculator.calculate(head);
        assert!(stats.z_mode);
        assert_eq!(stats.revision_index, 0);
        assert_eq!(stats.generation_index, 0);
        assert_eq!(stats.commit_index, 0);
    }
    #[test]
    fn test_merge_commit() {
        let mut repo = MockRepo::new(false);
        repo.add_commit("c0", vec![], None);
        repo.add_commit("c1", vec!["c0"], None);
        repo.add_commit("c2", vec!["c0"], None);
        let head_id = repo.add_commit("c3", vec!["c1", "c2"], None);
        let head = repo.commits.get(&head_id).unwrap();
        let calculator = GraphStatsCalculator::new(&repo, -1);
        let stats = calculator.calculate(head);
        assert_eq!(stats.revision_index, 2);
        assert_eq!(stats.generation_index, 2);
        assert_eq!(stats.commit_index, 3);
    }
    #[test]
    fn test_z_commit_not_trusted_during_initial_scan() {
        let mut repo = MockRepo::new(false);
        repo.add_commit("c0", vec![], None);
        repo.add_commit("c1", vec!["c0"], Some("z0 / x0000"));
        repo.add_commit("c2", vec!["c1"], None);
        let head_id = repo.add_commit("c3", vec!["c2"], None);
        let head = repo.commits.get(&head_id).unwrap();
        let calculator = GraphStatsCalculator::new(&repo, 5);
        let stats = calculator.calculate(head);
        assert!(!stats.z_mode);
        assert_eq!(stats.revision_index, 3);
    }
    #[test]
    fn test_z_commit_trusted_after_entering_z_mode() {
        let mut repo = MockRepo::new(false);
        repo.add_commit("c0", vec![], None);
        repo.add_commit("c1", vec!["c0"], None);
        repo.add_commit("c2", vec!["c1"], Some("z1 / x1111 / o1234"));
        repo.add_commit("c3", vec!["c2"], None);
        repo.add_commit("c4", vec!["c3"], None);
        repo.add_commit("c5", vec!["c4"], None);
        let head_id = repo.add_commit("c6", vec!["c5"], None);
        let head = repo.commits.get(&head_id).unwrap();
        let calculator = GraphStatsCalculator::new(&repo, 2);
        let stats = calculator.calculate(head);
        assert!(stats.z_mode);
        assert_eq!(stats.revision_index, 2);
    }
}
