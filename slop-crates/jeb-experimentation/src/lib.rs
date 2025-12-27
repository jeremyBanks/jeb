use {
    futures::Stream,
    parking_lot::Mutex,
    std::{
        pin::Pin,
        sync::Arc,
        task::{Context, Poll, Waker},
    },
};

/// Shared state between the two output streams.
struct SharedState<S, T, E>
where
    S: Stream<Item = Result<T, E>>,
{
    /// The input stream, None when exhausted.
    input: Option<Pin<Box<S>>>,
    /// Single-slot buffer for handoff between streams.
    buffer: Option<Result<T, E>>,
    /// Waker for the Ok stream, to wake when an Ok is available.
    ok_waker: Option<Waker>,
    /// Waker for the Err stream, to wake when an Err is available.
    err_waker: Option<Waker>,
    /// True if the Ok stream has been dropped.
    ok_dropped: bool,
    /// True if the Err stream has been dropped.
    err_dropped: bool,
}

/// Stream that yields Ok values from a Result stream.
pub struct OkStream<S, T, E>
where
    S: Stream<Item = Result<T, E>>,
{
    shared: Arc<Mutex<SharedState<S, T, E>>>,
}

/// Stream that yields Err values from a Result stream.
pub struct ErrStream<S, T, E>
where
    S: Stream<Item = Result<T, E>>,
{
    shared: Arc<Mutex<SharedState<S, T, E>>>,
}

impl<S, T, E> Drop for OkStream<S, T, E>
where
    S: Stream<Item = Result<T, E>>,
{
    fn drop(&mut self) {
        let mut state = self.shared.lock();
        state.ok_dropped = true;
        // Wake the err stream so it can make progress
        if let Some(waker) = state.err_waker.take() {
            waker.wake();
        }
    }
}

impl<S, T, E> Drop for ErrStream<S, T, E>
where
    S: Stream<Item = Result<T, E>>,
{
    fn drop(&mut self) {
        let mut state = self.shared.lock();
        state.err_dropped = true;
        // Wake the ok stream so it can make progress
        if let Some(waker) = state.ok_waker.take() {
            waker.wake();
        }
    }
}

/// Split a stream of `Result<T, E>` into two streams: one for `Ok` values, one for `Err` values.
///
/// This implementation:
/// - Does NOT spawn any tasks
/// - Does NOT depend on any async runtime
/// - Is pull-based (only does work when polled)
/// - Uses a single-element buffer for handoff
///
/// Both streams must be polled for the input to make progress. If one stream
/// encounters an item for the other, it buffers it and returns `Pending`.
pub fn oks_and_errs<S, T, E>(input: S) -> (OkStream<S, T, E>, ErrStream<S, T, E>)
where
    S: Stream<Item = Result<T, E>>,
{
    let shared = Arc::new(Mutex::new(SharedState {
        input: Some(Box::pin(input)),
        buffer: None,
        ok_waker: None,
        err_waker: None,
        ok_dropped: false,
        err_dropped: false,
    }));

    (
        OkStream {
            shared: Arc::clone(&shared),
        },
        ErrStream { shared },
    )
}

impl<S, T, E> Stream for OkStream<S, T, E>
where
    S: Stream<Item = Result<T, E>>,
{
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut state = self.shared.lock();

        loop {
            // First check the buffer
            match state.buffer.take() {
                Some(Ok(t)) => {
                    // It's for us! Wake the other stream so it can poll input.
                    if let Some(waker) = state.err_waker.take() {
                        waker.wake();
                    }
                    return Poll::Ready(Some(t));
                }
                Some(Err(e)) => {
                    // Not for us, put it back and wait
                    state.buffer = Some(Err(e));
                    state.ok_waker = Some(cx.waker().clone());
                    return Poll::Pending;
                }
                None => {
                    // Buffer is empty, try to poll input
                }
            }

            // Buffer is empty, poll the input stream
            match &mut state.input {
                None => {
                    // Input exhausted
                    return Poll::Ready(None);
                }
                Some(input) => {
                    match input.as_mut().poll_next(cx) {
                        Poll::Ready(Some(Ok(t))) => {
                            // Got an Ok, return it directly
                            return Poll::Ready(Some(t));
                        }
                        Poll::Ready(Some(Err(e))) => {
                            if state.err_dropped {
                                // Err stream is gone, discard and loop to get next
                                continue;
                            }
                            // Got an Err, buffer it for the other stream
                            state.buffer = Some(Err(e));
                            state.ok_waker = Some(cx.waker().clone());
                            // Wake the err stream
                            if let Some(waker) = state.err_waker.take() {
                                waker.wake();
                            }
                            return Poll::Pending;
                        }
                        Poll::Ready(None) => {
                            // Input exhausted
                            state.input = None;
                            // Wake err stream so it knows we're done
                            if let Some(waker) = state.err_waker.take() {
                                waker.wake();
                            }
                            return Poll::Ready(None);
                        }
                        Poll::Pending => {
                            // Input not ready, store our waker
                            state.ok_waker = Some(cx.waker().clone());
                            return Poll::Pending;
                        }
                    }
                }
            }
        }
    }
}

impl<S, T, E> Stream for ErrStream<S, T, E>
where
    S: Stream<Item = Result<T, E>>,
{
    type Item = E;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut state = self.shared.lock();

        loop {
            // First check the buffer
            match state.buffer.take() {
                Some(Err(e)) => {
                    // It's for us! Wake the other stream so it can poll input.
                    if let Some(waker) = state.ok_waker.take() {
                        waker.wake();
                    }
                    return Poll::Ready(Some(e));
                }
                Some(Ok(t)) => {
                    // Not for us, put it back and wait
                    state.buffer = Some(Ok(t));
                    state.err_waker = Some(cx.waker().clone());
                    return Poll::Pending;
                }
                None => {
                    // Buffer is empty, try to poll input
                }
            }

            // Buffer is empty, poll the input stream
            match &mut state.input {
                None => {
                    // Input exhausted
                    return Poll::Ready(None);
                }
                Some(input) => {
                    match input.as_mut().poll_next(cx) {
                        Poll::Ready(Some(Err(e))) => {
                            // Got an Err, return it directly
                            return Poll::Ready(Some(e));
                        }
                        Poll::Ready(Some(Ok(t))) => {
                            if state.ok_dropped {
                                // Ok stream is gone, discard and loop to get next
                                continue;
                            }
                            // Got an Ok, buffer it for the other stream
                            state.buffer = Some(Ok(t));
                            state.err_waker = Some(cx.waker().clone());
                            // Wake the ok stream
                            if let Some(waker) = state.ok_waker.take() {
                                waker.wake();
                            }
                            return Poll::Pending;
                        }
                        Poll::Ready(None) => {
                            // Input exhausted
                            state.input = None;
                            // Wake ok stream so it knows we're done
                            if let Some(waker) = state.ok_waker.take() {
                                waker.wake();
                            }
                            return Poll::Ready(None);
                        }
                        Poll::Pending => {
                            // Input not ready, store our waker
                            state.err_waker = Some(cx.waker().clone());
                            return Poll::Pending;
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::stream::{self, StreamExt};

    // Helper to block on a future without any runtime
    fn block_on<F: std::future::Future>(f: F) -> F::Output {
        use std::task::{RawWaker, RawWakerVTable};

        const VTABLE: RawWakerVTable = RawWakerVTable::new(
            |_| RawWaker::new(std::ptr::null(), &VTABLE),
            |_| {},
            |_| {},
            |_| {},
        );

        let waker = unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), &VTABLE)) };
        let mut cx = Context::from_waker(&waker);
        let mut f = std::pin::pin!(f);

        loop {
            match f.as_mut().poll(&mut cx) {
                Poll::Ready(val) => return val,
                Poll::Pending => {
                    // In a real scenario we'd park, but for simple tests
                    // where streams are ready, this works
                    std::hint::spin_loop();
                }
            }
        }
    }

    #[test]
    fn test_all_oks() {
        let input = stream::iter(vec![Ok::<_, &str>(1), Ok(2), Ok(3)]);
        let (ok_stream, _err_stream) = oks_and_errs(input);

        let results: Vec<i32> = block_on(ok_stream.collect());
        assert_eq!(results, vec![1, 2, 3]);
    }

    #[test]
    fn test_all_errs() {
        let input = stream::iter(vec![Err::<i32, _>("a"), Err("b"), Err("c")]);
        let (_ok_stream, err_stream) = oks_and_errs(input);

        let results: Vec<&str> = block_on(err_stream.collect());
        assert_eq!(results, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_drop_ok_stream() {
        let input = stream::iter(vec![Ok::<_, &str>(1), Err("a"), Ok(2), Err("b")]);
        let (ok_stream, err_stream) = oks_and_errs(input);

        // Drop the ok stream
        drop(ok_stream);

        // Err stream should still get all errors
        let results: Vec<&str> = block_on(err_stream.collect());
        assert_eq!(results, vec!["a", "b"]);
    }

    #[test]
    fn test_drop_err_stream() {
        let input = stream::iter(vec![Ok::<_, &str>(1), Err("a"), Ok(2), Err("b")]);
        let (ok_stream, err_stream) = oks_and_errs(input);

        // Drop the err stream
        drop(err_stream);

        // Ok stream should still get all oks
        let results: Vec<i32> = block_on(ok_stream.collect());
        assert_eq!(results, vec![1, 2]);
    }
}
