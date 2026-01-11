use std::path::PathBuf;

use clap::{Parser, Subcommand};
use git_snapshot::{CommitIdStyle, Repository, SerializationOptions};

#[derive(Parser)]
#[clap(name = "git-snapshot")]
#[clap(about = "Capture and expand git repository snapshots")]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Capture a git repository to a YAML snapshot file
    Capture {
        /// Path to the git repository (defaults to current directory)
        #[clap(long, default_value = ".")]
        repo: PathBuf,

        /// Path to write the snapshot file
        #[clap(long)]
        snapshot: PathBuf,

        /// Use integer IDs instead of hex hashes
        #[clap(long)]
        integer_ids: bool,

        /// Include all optional fields in output
        #[clap(long)]
        verbose: bool,
    },

    /// Expand a YAML snapshot file to a git repository
    Expand {
        /// Path to the snapshot file
        #[clap(long)]
        snapshot: PathBuf,

        /// Path to create the git repository
        #[clap(long)]
        repo: PathBuf,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Command::Capture {
            repo,
            snapshot,
            integer_ids,
            verbose,
        } => {
            let git_path = if repo.join(".git").exists() {
                repo.join(".git")
            } else {
                repo
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

            eprintln!("Captured {} to {}", git_path.display(), snapshot.display());
        }

        Command::Expand { snapshot, repo } => {
            let yaml = std::fs::read_to_string(&snapshot)?;
            let repository = git_snapshot::parse(&yaml)?;

            repository.to_repository_at_path(&repo)?;

            eprintln!(
                "Expanded {} to {}",
                snapshot.display(),
                repo.display()
            );
        }
    }

    Ok(())
}
