use {
    crate::Item,
    futures::{Stream, StreamExt},
    std::{path::Path, pin::pin},
    tokio::io::AsyncWriteExt,
};

/// Consumes a stream and writes items to stdout.
///
/// Handles `Item::Bytes` and `Item::Text`, panicking on other variants.
///
/// # Panics
///
/// Panics if the stream contains non-text/non-bytes items, or if writing fails.
#[cfg(feature = "stdio")]
pub async fn stdout<S>(input: S)
where
    S: Stream<Item = Item> + Send,
{
    let mut input = pin!(input);
    let mut writer = tokio::io::stdout();

    while let Some(item) = input.next().await {
        match item {
            Item::Bytes(bytes) => {
                writer
                    .write_all(&bytes)
                    .await
                    .expect("failed to write bytes");
            }
            Item::Text(text) => {
                writer
                    .write_all(text.as_bytes())
                    .await
                    .expect("failed to write text");
            }
            _ => panic!("stdout received non-text/non-bytes item"),
        }
    }
}

/// Consumes a stream and writes items to stderr.
///
/// Handles `Item::Bytes` and `Item::Text`, panicking on other variants.
///
/// # Panics
///
/// Panics if the stream contains non-text/non-bytes items, or if writing fails.
#[cfg(feature = "stdio")]
pub async fn stderr<S>(input: S)
where
    S: Stream<Item = Item> + Send,
{
    let mut input = pin!(input);
    let mut writer = tokio::io::stderr();

    while let Some(item) = input.next().await {
        match item {
            Item::Bytes(bytes) => {
                writer
                    .write_all(&bytes)
                    .await
                    .expect("failed to write bytes");
            }
            Item::Text(text) => {
                writer
                    .write_all(text.as_bytes())
                    .await
                    .expect("failed to write text");
            }
            _ => panic!("stderr received non-text/non-bytes item"),
        }
    }
}

/// Consumes a stream and writes items to a file.
///
/// Handles `Item::Bytes` and `Item::Text`, panicking on other variants.
///
/// # Panics
///
/// Panics if the file cannot be created, if the stream contains non-text/non-bytes
/// items, or if writing fails.
#[cfg(feature = "fs")]
pub async fn write_path<S, P>(input: S, path: P)
where
    S: Stream<Item = Item> + Send,
    P: AsRef<Path>,
{
    let mut input = pin!(input);
    let mut writer = tokio::fs::File::create(path)
        .await
        .expect("failed to create file");

    while let Some(item) = input.next().await {
        match item {
            Item::Bytes(bytes) => {
                writer
                    .write_all(&bytes)
                    .await
                    .expect("failed to write bytes");
            }
            Item::Text(text) => {
                writer
                    .write_all(text.as_bytes())
                    .await
                    .expect("failed to write text");
            }
            _ => panic!("write_path received non-text/non-bytes item"),
        }
    }
}
