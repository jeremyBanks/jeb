mod tasks;
mod workflows;

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[clap(name = "_ci", about = "Type-safe CI tasks and workflow generation")]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run build checks (cargo build + publish dry-run + deno test)
    Build,
    /// Run build with warnings-as-errors
    BuildWarnings,
    /// Run all tests
    Test,
    /// Run all CI checks (build + build-warnings + test)
    All,
    /// Generate GitHub Actions workflow files
    GenerateWorkflows,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build => tasks::build()?,
        Commands::BuildWarnings => tasks::build_warnings()?,
        Commands::Test => tasks::test()?,
        Commands::All => {
            tasks::build()?;
            tasks::build_warnings()?;
            tasks::test()?;
        }
        Commands::GenerateWorkflows => workflows::generate_all()?,
    }

    Ok(())
}
