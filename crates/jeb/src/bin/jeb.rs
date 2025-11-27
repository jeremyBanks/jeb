use {
    jeb::{
        Panic,
        pipeline::{Executor, Pipeline},
    },
    owo_colors::OwoColorize,
    std::convert::Infallible,
};


#[tokio::main(flavor = "current_thread")]
pub async fn main() -> Result<(), Infallible> {
    let exit_code = inner_main().await.unwrap_or(1);
    std::process::exit(i32::from(exit_code));
}

pub async fn inner_main() -> Result<u8, Panic> {
    let mut args = Vec::<String>::from_iter(std::env::args());
    let own_path: String = args.remove(0);

    let commands = args;

    // Parse the pipeline
    let pipeline = Pipeline::parse(&commands);

    // Display the pipeline
    let display = pipeline.display();
    eprintln!("{} {}", own_path.magenta(), display.yellow());

    // Execute the pipeline
    let mut executor = Executor::new();
    let result = executor.execute(&pipeline);

    // Print warnings to stderr
    for warning in &result.warnings {
        eprintln!(
            "{}: {}",
            format!("warning[{}]", warning.node_index).yellow(),
            warning.message
        );
    }

    // Print errors to stderr
    for error in &result.errors {
        eprintln!(
            "{}: {}",
            format!("error[{}]", error.node_index).red(),
            error.message
        );
        if let Some(ctx) = &error.context {
            eprintln!("  {}: {}", "context".dimmed(), ctx.dimmed());
        }
    }

    Ok(result.exit_status)
}
