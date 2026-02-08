//! ZIP example - creates a simple ZIP archive.

use std::fs;
use zipng::{panic, Zip};

fn main() -> Result<(), panic> {
    fs::create_dir_all("target")?;

    let zip = Zip::new_with_files(vec![
        (b"README.md".to_vec(), b"hello, world?".to_vec()),
        (b"LICENSE.md".to_vec(), b"not applicable".to_vec()),
    ]);

    let output = zip.serialize();
    fs::write("target/zip_example.zip", AsRef::<[u8]>::as_ref(&output))?;
    println!("Created target/zip_example.zip ({} bytes)", output.len());
    println!("Verify with: unzip -l target/zip_example.zip");

    Ok(())
}

#[test]
fn test() {
    main().unwrap()
}
