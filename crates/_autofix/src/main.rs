use anyhow::Result;

mod workspace_deps;

fn main() -> Result<()> {
    workspace_deps::main()
}
