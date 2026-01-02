use crate::command_runner::run_command;
pub fn main() -> i32 {
    run_command("deno", &["lint", "--fix"])
}
