use std::process::Command;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    // Parse arguments
    let mut message = None;
    let mut allow_empty = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-m" => {
                i += 1;
                if i < args.len() {
                    message = Some(args[i].clone());
                }
            }
            "--allow-empty" | "--empty" => {
                allow_empty = true;
            }
            _ => {}
        }
        i += 1;
    }

    // Stage all changes
    let status = Command::new("git")
        .args(["add", "-A"])
        .status()
        .expect("failed to run git add");
    if !status.success() {
        std::process::exit(1);
    }

    // Build commit args
    let mut commit_args = vec!["commit".to_string()];

    if allow_empty {
        commit_args.push("--allow-empty".to_string());
    }

    let msg = message.unwrap_or_else(|| "wip".to_string());
    commit_args.push("-m".to_string());
    commit_args.push(msg);

    let status = Command::new("git")
        .args(&commit_args)
        .status()
        .expect("failed to run git commit");

    if !status.success() {
        // If commit fails (e.g., nothing to commit), that's OK
        if !allow_empty {
            eprintln!("(nothing to commit)");
        }
    }
}
