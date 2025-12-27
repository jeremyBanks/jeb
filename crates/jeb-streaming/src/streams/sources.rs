use {
    crate::Item,
    async_stream::stream,
    futures::Stream,
    std::path::Path,
    tokio::io::AsyncReadExt,
};

/// Creates a stream that reads from stdin.
///
/// Returns a stream of `Result<Item, &'static str>` where items are `Item::Bytes`.
#[cfg(feature = "stdio")]
pub fn stdin() -> impl Stream<Item = Result<Item, &'static str>> + Send + Unpin {
    Box::pin(stream! {
        let mut reader = tokio::io::stdin();
        let mut buffer = [0u8; 65_536];

        loop {
            match reader.read(&mut buffer).await {
                Ok(0) => break,
                Ok(n) => yield Ok(Item::Bytes(buffer[..n].to_vec().into())),
                Err(_) => {
                    yield Err("failed to read stdin");
                    break;
                }
            }
        }
    })
}

/// Creates a stream that reads from a file path.
///
/// Returns a stream of `Result<Item, &'static str>` where items are `Item::Bytes`.
#[cfg(feature = "fs")]
pub fn read_path<P>(path: P) -> impl Stream<Item = Result<Item, &'static str>> + Send + Unpin
where
    P: AsRef<Path> + Send + 'static,
{
    Box::pin(stream! {
        match tokio::fs::File::open(path).await {
            Ok(mut reader) => {
                let mut buffer = [0u8; 65_536];

                loop {
                    match reader.read(&mut buffer).await {
                        Ok(0) => break,
                        Ok(n) => yield Ok(Item::Bytes(buffer[..n].to_vec().into())),
                        Err(_) => {
                            yield Err("failed to read");
                            break;
                        }
                    }
                }
            }
            Err(_) => {
                yield Err("failed to open file");
            }
        }
    })
}

/// Creates a stream from an iterator of strings.
///
/// Returns a stream of `Result<Item, &'static str>` where items are `Item::Text`.
pub fn text_source<IntoTextIterator>(
    text: IntoTextIterator,
) -> impl Stream<Item = Result<Item, &'static str>> + Send
where
    IntoTextIterator: IntoIterator<Item = String> + Send + 'static,
    IntoTextIterator::IntoIter: Send,
{
    stream! {
        for text_item in text {
            yield Ok(Item::Text(text_item.into()));
        }
    }
}

/// Creates a stream from an iterator of byte vectors.
///
/// Returns a stream of `Result<Item, &'static str>` where items are `Item::Bytes`.
pub fn bytes_source<IntoBytesIterator>(
    bytes: IntoBytesIterator,
) -> impl Stream<Item = Result<Item, &'static str>> + Send
where
    IntoBytesIterator: IntoIterator<Item = Vec<u8>> + Send + 'static,
    IntoBytesIterator::IntoIter: Send,
{
    stream! {
        for bytes_item in bytes {
            yield Ok(Item::Bytes(bytes_item.into()));
        }
    }
}
