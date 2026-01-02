use {
    async_stream::stream,
    futures::{
        Stream,
        StreamExt,
    },
    std::pin::pin,
};

/// Filters a Result stream to only yield Ok values, discarding Err values.
///
/// This is a simpler alternative to `oks_and_errs()` when you don't need
/// the error stream.
pub fn oks<S, T, E>(input: S) -> impl Stream<Item = T> + Send
where
    S: Stream<Item = Result<T, E>> + Send + 'static,
    T: Send,
    E: Send,
{
    stream! {
        let mut input = pin!(input);
        while let Some(item) = input.next().await {
            if let Ok(value) = item {
                yield value;
            }
        }
    }
}

/// Filters a Result stream to only yield Err values, discarding Ok values.
///
/// This is a simpler alternative to `oks_and_errs()` when you don't need
/// the ok stream.
pub fn errs<S, T, E>(input: S) -> impl Stream<Item = E> + Send
where
    S: Stream<Item = Result<T, E>> + Send + 'static,
    T: Send,
    E: Send,
{
    stream! {
        let mut input = pin!(input);
        while let Some(item) = input.next().await {
            if let Err(error) = item {
                yield error;
            }
        }
    }
}

/// Unwraps Ok values from a Result stream, panicking on Err values.
///
/// # Panics
///
/// Panics if the stream yields an Err value.
pub fn unwrap_oks<S, T, E>(input: S) -> impl Stream<Item = T> + Send
where
    S: Stream<Item = Result<T, E>> + Send + 'static,
    T: Send,
    E: Send + std::fmt::Debug,
{
    stream! {
        let mut input = pin!(input);
        while let Some(item) = input.next().await {
            yield item.unwrap();
        }
    }
}

/// Yields Ok values from a Result stream until the first Err is encountered.
///
/// Once an Err is encountered, the stream terminates without yielding the
/// error.
pub fn fail_fast<S, T, E>(input: S) -> impl Stream<Item = T> + Send
where
    S: Stream<Item = Result<T, E>> + Send + 'static,
    T: Send,
    E: Send,
{
    stream! {
        let mut input = pin!(input);
        while let Some(item) = input.next().await {
            match item {
                Ok(value) => yield value,
                Err(_) => break,
            }
        }
    }
}
