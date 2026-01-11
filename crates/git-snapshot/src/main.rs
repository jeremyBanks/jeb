use std::path::PathBuf;

use clap::{Parser, Subcommand};
use git_snapshot::{CommitIdStyle, Repository, SerializationOptions, Tree};

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
        #[clap(long, required_unless_present = "dir", conflicts_with = "dir")]
        repo: Option<PathBuf>,

        /// Path to a plain directory to capture as working tree only (mutually exclusive with --repo)
        #[clap(long, required_unless_present = "repo", conflicts_with = "repo")]
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
        #[clap(long, required_unless_present = "dir", conflicts_with = "dir")]
        repo: Option<PathBuf>,

        /// Path to extract working tree only as plain directory (mutually exclusive with --repo)
        #[clap(long, required_unless_present = "repo", conflicts_with = "repo")]
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
        if path
            .components()
            .any(|c| c.as_os_str() == ".git")
        {
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

        Command::Expand { snapshot, repo, dir } => {
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
