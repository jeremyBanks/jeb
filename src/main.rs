use clap::{Parser, Subcommand};
use jeb::{calculate, entity_from_json, entity_to_json, greet, process_data, Entity};
use tracing::{error, info};
use tracing_subscriber;

/// JSON Entity Bucket - A demo Rust application
#[derive(Parser, Debug)]
#[command(name = "jeb")]
#[command(about = "JSON Entity Bucket CLI", long_about = None)]
struct Cli {
    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Greet someone
    Greet {
        /// Name to greet
        #[arg(default_value = "World")]
        name: String,
    },
    /// Calculate sum of two numbers
    Calculate {
        /// First number
        x: i32,
        /// Second number
        y: i32,
    },
    /// Process some data
    Process {
        /// Data as comma-separated bytes (e.g., "1,2,3,4,5")
        #[arg(default_value = "1,2,3,4,5")]
        data: String,
    },
    /// Create and display an entity as JSON
    Entity {
        /// Entity ID
        #[arg(short, long)]
        id: i64,
        /// Entity name
        #[arg(short, long)]
        name: String,
        /// Optional metadata
        #[arg(short, long)]
        metadata: Option<String>,
    },
    /// Parse an entity from JSON
    ParseEntity {
        /// JSON string representing an entity
        json: String,
    },
}

fn main() {
    let cli = Cli::parse();

    // Initialize tracing
    let filter = if cli.debug {
        "debug"
    } else {
        "info"
    };
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .init();

    info!("JEB starting...");

    match cli.command {
        Commands::Greet { name } => {
            let greeting = greet(&name);
            println!("{}", greeting);
        }
        Commands::Calculate { x, y } => {
            let result = calculate(x, y);
            println!("{} + {} = {}", x, y, result);
        }
        Commands::Process { data } => {
            let bytes: Vec<u8> = data
                .split(',')
                .filter_map(|s| s.trim().parse().ok())
                .collect();
            let processed = process_data(&bytes);
            println!("Input:  {:?}", bytes);
            println!("Output: {:?}", processed);
        }
        Commands::Entity { id, name, metadata } => {
            let mut entity = Entity::new(id, name);
            if let Some(meta) = metadata {
                entity = entity.with_metadata(meta);
            }

            match entity_to_json(&entity) {
                Ok(json) => println!("{}", json),
                Err(e) => {
                    error!("Failed to serialize entity: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::ParseEntity { json } => {
            match entity_from_json(&json) {
                Ok(entity) => {
                    println!("Parsed entity:");
                    println!("  ID: {}", entity.id);
                    println!("  Name: {}", entity.name);
                    println!("  Metadata: {:?}", entity.metadata);
                }
                Err(e) => {
                    error!("Failed to parse entity: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }

    info!("JEB completed successfully");
}
