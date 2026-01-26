use zipng::{
    dev::{init, save},
    panic, Zip,
};

fn main() -> Result<(), panic> {
    init!();

    let zip = Zip::new_with_files(vec![
        (b"README.md".to_vec(), b"hello, world?".to_vec()),
        (b"LICENSE.md".to_vec(), b"not applicable".to_vec()),
    ]);

    let output = zip.serialize();
    save!({ output.as_ref() }.zip)?;

    let text = include_bytes!("../src/ss.htm");
    let mut text = text.to_vec();
    text.extend(output.into_bytes());

    save!(text.htm)?;
    Ok(())
}

#[test]
fn test() {
    main().unwrap()
}
