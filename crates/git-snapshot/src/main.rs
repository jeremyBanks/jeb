use {
    clap::{
        Parser,
        Subcommand,
    },
    git_snapshot::{
        CommitIdStyle,
        Repository,
        SerializationOptions,
        Tree,
    },
    std::path::PathBuf,
};

#[derive(Parser)]
#[clap(name = "git-snapshot")]
#[clap(about = "Capture and expand git repository snapshots")]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Capture a git repository or directory to a YAML snapshot file
    Capture {
        /// Path to the git repository (mutually exclusive with --dir)
        #[clap(
            long,
            required_unless_present = "dir",
            conflicts_with = "dir"
        )]
        repo: Option<PathBuf>,

        /// Path to a plain directory to capture as working tree only (mutually
        /// exclusive with --repo)
        #[clap(
            long,
            required_unless_present = "repo",
            conflicts_with = "repo"
        )]
        dir: Option<PathBuf>,

        /// Path to write the snapshot file
        #[clap(long)]
        snapshot: PathBuf,

        /// Use integer IDs instead of hex hashes (only applies to --repo mode)
        #[clap(long)]
        integer_ids: bool,

        /// Include all optional fields in output (only applies to --repo mode)
        #[clap(long)]
        verbose: bool,
    },

    /// Expand a YAML snapshot file to a git repository or directory
    Expand {
        /// Path to the snapshot file
        #[clap(long)]
        snapshot: PathBuf,

        /// Path to create the git repository (mutually exclusive with --dir)
        #[clap(
            long,
            required_unless_present = "dir",
            conflicts_with = "dir"
        )]
        repo: Option<PathBuf>,

        /// Path to extract working tree only as plain directory (mutually
        /// exclusive with --repo)
        #[clap(
            long,
            required_unless_present = "repo",
            conflicts_with = "repo"
        )]
        dir: Option<PathBuf>,
    },
}

fn capture_directory(dir: &PathBuf) -> Result<Repository, Box<dyn std::error::Error>> {
    use ignore::WalkBuilder;

    let mut tree = Tree::new();

    let walker = WalkBuilder::new(dir)
        .hidden(false) // include hidden files
        .git_ignore(true) // respect .gitignore
        .git_global(false) // don't use global gitignore
        .git_exclude(true) // respect .git/info/exclude if present
        .require_git(false) // don't require a git repo
        .build();

    for entry in walker {
        let entry = entry?;
        let path = entry.path();

        // Skip the root directory itself
        if path == dir.as_path() {
            continue;
        }

        // Skip .git directory
        if path.components().any(|c| c.as_os_str() == ".git") {
            continue;
        }

        // Skip directories (we only care about files)
        if !path.is_file() {
            continue;
        }

        // Get relative path
        let relative = path
            .strip_prefix(dir)
            .map_err(|e| format!("Failed to get relative path: {}", e))?;

        let relative_str = relative
            .to_str()
            .ok_or_else(|| format!("Non-UTF8 path: {:?}", relative))?;

        // Read file contents
        let contents = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", relative_str, e))?;

        tree.insert(relative_str.to_string(), contents);
    }

    let mut repository = Repository::new();
    repository.set_working(Some(tree));

    Ok(repository)
}

fn expand_to_directory(
    repository: &Repository,
    dir: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let working = repository
        .working()
        .ok_or("Snapshot has no working tree to expand")?;

    std::fs::create_dir_all(dir)?;

    for path in working.paths() {
        let content = working.get(path).expect("path exists");
        let file_path = dir.join(path);

        // Ensure parent directory exists
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(&file_path, content)?;
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Command::Capture {
            repo,
            dir,
            snapshot,
            integer_ids,
            verbose,
        } => {
            if let Some(repo_path) = repo {
                // Capture from git repository
                let git_path = if repo_path.join(".git").exists() {
                    repo_path.join(".git")
                } else {
                    repo_path.clone()
                };

                let repository = Repository::from_git_dir(&git_path)?;

                let id_style = if integer_ids {
                    CommitIdStyle::Integer
                } else {
                    CommitIdStyle::Hex
                };

                let options = SerializationOptions {
                    include_all_fields: verbose,
                    ..Default::default()
                };

                let yaml = git_snapshot::serialize(&repository, id_style, options);
                std::fs::write(&snapshot, &yaml)?;

                eprintln!("Captured {} to {}", repo_path.display(), snapshot.display());
            } else if let Some(dir_path) = dir {
                // Capture from plain directory
                let repository = capture_directory(&dir_path)?;

                // For directory mode, we just serialize the working tree
                let yaml = git_snapshot::serialize(
                    &repository,
                    CommitIdStyle::Integer, // doesn't matter, no commits
                    SerializationOptions::default(),
                );
                std::fs::write(&snapshot, &yaml)?;

                eprintln!("Captured {} to {}", dir_path.display(), snapshot.display());
            }
        }

        Command::Expand {
            snapshot,
            repo,
            dir,
        } => {
            let yaml = std::fs::read_to_string(&snapshot)?;
            let repository = git_snapshot::parse(&yaml)?;

            if let Some(repo_path) = repo {
                // Expand to git repository
                repository.to_repository_at_path(&repo_path)?;

                eprintln!("Expanded {} to {}", snapshot.display(), repo_path.display());
            } else if let Some(dir_path) = dir {
                // Expand working tree only to plain directory
                expand_to_directory(&repository, &dir_path)?;

                eprintln!("Expanded {} to {}", snapshot.display(), dir_path.display());
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        std::collections::BTreeSet,
    };

    fn create_test_dir() -> tempfile::TempDir {
        tempfile::TempDir::new().expect("Failed to create temp dir")
    }

    fn write_file(dir: &std::path::Path, path: &str, content: &str) {
        let file_path = dir.join(path);
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent).expect("Failed to create parent dirs");
        }
        std::fs::write(file_path, content).expect("Failed to write file");
    }

    fn read_file(dir: &std::path::Path, path: &str) -> String {
        std::fs::read_to_string(dir.join(path)).expect("Failed to read file")
    }

    #[test]
    fn test_capture_directory_basic() {
        let dir = create_test_dir();
        write_file(dir.path(), "README.md", "# Hello");
        write_file(dir.path(), "src/main.rs", "fn main() {}");
        write_file(dir.path(), "src/lib.rs", "pub fn foo() {}");

        let repo = capture_directory(&dir.path().to_path_buf()).unwrap();
        let working = repo.working().expect("should have working tree");

        assert_eq!(working.get("README.md"), Some("# Hello"));
        assert_eq!(working.get("src/main.rs"), Some("fn main() {}"));
        assert_eq!(working.get("src/lib.rs"), Some("pub fn foo() {}"));
    }

    #[test]
    fn test_capture_directory_empty() {
        let dir = create_test_dir();

        let repo = capture_directory(&dir.path().to_path_buf()).unwrap();
        let working = repo.working().expect("should have working tree");

        assert!(working.is_empty());
    }

    #[test]
    fn test_capture_directory_skips_git_folder() {
        let dir = create_test_dir();
        write_file(dir.path(), "README.md", "# Hello");
        write_file(dir.path(), ".git/config", "[core]");
        write_file(dir.path(), ".git/objects/abc", "blob data");

        let repo = capture_directory(&dir.path().to_path_buf()).unwrap();
        let working = repo.working().expect("should have working tree");

        let paths: BTreeSet<_> = working.paths().collect();
        assert!(paths.contains("README.md"));
        assert!(!paths.iter().any(|p| p.contains(".git")));
    }

    #[test]
    fn test_capture_directory_includes_hidden_files() {
        let dir = create_test_dir();
        write_file(dir.path(), ".hidden", "secret");
        write_file(dir.path(), ".config/settings", "value=1");

        let repo = capture_directory(&dir.path().to_path_buf()).unwrap();
        let working = repo.working().expect("should have working tree");

        assert_eq!(working.get(".hidden"), Some("secret"));
        assert_eq!(working.get(".config/settings"), Some("value=1"));
    }

    #[test]
    fn test_capture_directory_respects_gitignore() {
        let dir = create_test_dir();
        write_file(dir.path(), ".gitignore", "*.log\ntarget/\n");
        write_file(dir.path(), "README.md", "# Hello");
        write_file(dir.path(), "debug.log", "log content");
        write_file(dir.path(), "target/debug/main", "binary");

        let repo = capture_directory(&dir.path().to_path_buf()).unwrap();
        let working = repo.working().expect("should have working tree");

        let paths: BTreeSet<_> = working.paths().collect();
        assert!(paths.contains("README.md"));
        assert!(paths.contains(".gitignore"));
        assert!(!paths.contains("debug.log"));
        assert!(!paths.iter().any(|p| p.starts_with("target/")));
    }

    #[test]
    fn test_capture_directory_nested_gitignore() {
        let dir = create_test_dir();
        write_file(dir.path(), "README.md", "# Hello");
        write_file(dir.path(), "src/.gitignore", "*.bak\n");
        write_file(dir.path(), "src/main.rs", "fn main() {}");
        write_file(dir.path(), "src/main.rs.bak", "old version");
        write_file(dir.path(), "docs/notes.bak", "should exist"); // not in src/

        let repo = capture_directory(&dir.path().to_path_buf()).unwrap();
        let working = repo.working().expect("should have working tree");

        let paths: BTreeSet<_> = working.paths().collect();
        assert!(paths.contains("src/main.rs"));
        assert!(!paths.contains("src/main.rs.bak"));
        assert!(paths.contains("docs/notes.bak")); // nested gitignore doesn't apply here
    }

    #[test]
    fn test_expand_to_directory_basic() {
        let mut repo = Repository::new();
        let mut tree = Tree::new();
        tree.insert("README.md".to_string(), "# Hello".to_string());
        tree.insert("src/main.rs".to_string(), "fn main() {}".to_string());
        repo.set_working(Some(tree));

        let dir = create_test_dir();
        expand_to_directory(&repo, &dir.path().to_path_buf()).unwrap();

        assert_eq!(read_file(dir.path(), "README.md"), "# Hello");
        assert_eq!(read_file(dir.path(), "src/main.rs"), "fn main() {}");
    }

    #[test]
    fn test_expand_to_directory_creates_nested_dirs() {
        let mut repo = Repository::new();
        let mut tree = Tree::new();
        tree.insert("a/b/c/d/file.txt".to_string(), "deep".to_string());
        repo.set_working(Some(tree));

        let dir = create_test_dir();
        expand_to_directory(&repo, &dir.path().to_path_buf()).unwrap();

        assert_eq!(read_file(dir.path(), "a/b/c/d/file.txt"), "deep");
    }

    #[test]
    fn test_expand_to_directory_no_working_tree_errors() {
        let repo = Repository::new();
        let dir = create_test_dir();

        let result = expand_to_directory(&repo, &dir.path().to_path_buf());
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("no working tree"));
    }

    #[test]
    fn test_roundtrip_capture_expand() {
        // Create source directory with files
        let source = create_test_dir();
        write_file(source.path(), "README.md", "# Project");
        write_file(source.path(), "src/lib.rs", "pub mod foo;");
        write_file(source.path(), "src/foo.rs", "pub fn bar() {}");
        write_file(source.path(), ".hidden", "secret");

        // Capture
        let repo = capture_directory(&source.path().to_path_buf()).unwrap();

        // Expand to new directory
        let dest = create_test_dir();
        expand_to_directory(&repo, &dest.path().to_path_buf()).unwrap();

        // Verify contents match
        assert_eq!(read_file(dest.path(), "README.md"), "# Project");
        assert_eq!(read_file(dest.path(), "src/lib.rs"), "pub mod foo;");
        assert_eq!(read_file(dest.path(), "src/foo.rs"), "pub fn bar() {}");
        assert_eq!(read_file(dest.path(), ".hidden"), "secret");
    }

    #[test]
    fn test_roundtrip_via_yaml() {
        // Create source directory
        let source = create_test_dir();
        write_file(source.path(), "file.txt", "content");
        write_file(source.path(), "nested/deep/file.md", "# Nested");

        // Capture to YAML
        let repo = capture_directory(&source.path().to_path_buf()).unwrap();
        let yaml = git_snapshot::serialize(
            &repo,
            CommitIdStyle::Integer,
            SerializationOptions::default(),
        );

        // Parse YAML and expand
        let parsed = git_snapshot::parse(&yaml).unwrap();
        let dest = create_test_dir();
        expand_to_directory(&parsed, &dest.path().to_path_buf()).unwrap();

        // Verify
        assert_eq!(read_file(dest.path(), "file.txt"), "content");
        assert_eq!(read_file(dest.path(), "nested/deep/file.md"), "# Nested");
    }

    #[test]
    fn test_roundtrip_preserves_all_files() {
        let source = create_test_dir();
        let files = vec![
            ("a.txt", "a"),
            ("b.txt", "b"),
            ("dir1/c.txt", "c"),
            ("dir1/dir2/d.txt", "d"),
            (".dotfile", "dot"),
            (".dotdir/file", "dotdir file"),
        ];

        for (path, content) in &files {
            write_file(source.path(), path, content);
        }

        let repo = capture_directory(&source.path().to_path_buf()).unwrap();
        let dest = create_test_dir();
        expand_to_directory(&repo, &dest.path().to_path_buf()).unwrap();

        for (path, content) in &files {
            assert_eq!(
                read_file(dest.path(), path),
                *content,
                "Mismatch for {}",
                path
            );
        }
    }
}
