use std::process::Command;

fn main() {
    // Find zipng binary next to this binary
    let zipng = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("zipng")))
        .unwrap_or_else(|| "zipng".into());

    let args: Vec<_> = std::env::args_os().skip(1).collect();

    let mut cmd = Command::new(&zipng);
    cmd.arg("--extract");
    cmd.args(&args);

    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let err = cmd.exec();
        eprintln!("unzipng: {err}");
        std::process::exit(1);
    }

    #[cfg(not(unix))]
    {
        match cmd.status() {
            Ok(status) => std::process::exit(status.code().unwrap_or(1)),
            Err(err) => {
                eprintln!("unzipng: {err}");
                std::process::exit(1);
            }
        }
    }
}
