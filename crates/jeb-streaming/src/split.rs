use {
    futures::{
        Stream,
        stream::FusedStream,
    },
    parking_lot::Mutex,
    std::{
        pin::Pin,
        sync::Arc,
        task::{
            Context,
            Poll,
            Waker,
        },
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

impl<S, T, E> Unpin for OkStream<S, T, E> where S: Stream<Item = Result<T, E>> {}

/// Stream that yields Err values from a Result stream.
pub struct ErrStream<S, T, E>
where
    S: Stream<Item = Result<T, E>>,
{
    shared: Arc<Mutex<SharedState<S, T, E>>>,
}

impl<S, T, E> Unpin for ErrStream<S, T, E> where S: Stream<Item = Result<T, E>> {}

impl<S, T, E> Drop for OkStream<S, T, E>
where
    S: Stream<Item = Result<T, E>>,
{
    fn drop(&mut self) {
        // Extract waker before releasing lock to avoid deadlock
        let waker = {
            let mut state = self.shared.lock();
            state.ok_dropped = true;
            state.err_waker.take()
        }; // Lock released here

        // Wake the err stream so it can make progress (outside lock)
        if let Some(waker) = waker {
            waker.wake();
        }
    }
}

impl<S, T, E> Drop for ErrStream<S, T, E>
where
    S: Stream<Item = Result<T, E>>,
{
    fn drop(&mut self) {
        // Extract waker before releasing lock to avoid deadlock
        let waker = {
            let mut state = self.shared.lock();
            state.err_dropped = true;
            state.ok_waker.take()
        }; // Lock released here

        // Wake the ok stream so it can make progress (outside lock)
        if let Some(waker) = waker {
            waker.wake();
        }
    }
}

/// Split a stream of `Result<T, E>` into two streams: one for `Ok` values, one
/// for `Err` values.
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

        // First check the buffer
        match state.buffer.take() {
            Some(Ok(t)) => {
                // It's for us! Wake the other stream so it can poll input.
                let waker = state.err_waker.take();
                drop(state); // Release lock before waking
                if let Some(waker) = waker {
                    waker.wake();
                }
                return Poll::Ready(Some(t));
            }
            Some(Err(e)) => {
                // Not for us - check if the other stream was dropped
                if state.err_dropped {
                    // Err stream is gone, discard this item and continue
                    // polling input Fall through to input
                    // polling section
                } else {
                    // Put it back and wait for err stream to consume it
                    state.buffer = Some(Err(e));
                    // Only clone waker if it's different from current
                    if !state
                        .ok_waker
                        .as_ref()
                        .is_some_and(|w| w.will_wake(cx.waker()))
                    {
                        state.ok_waker = Some(cx.waker().clone());
                    }
                    // Re-wake the err stream in case of spurious wakeup
                    let waker = state.err_waker.take();
                    drop(state); // Release lock before waking
                    if let Some(waker) = waker {
                        waker.wake();
                    }
                    return Poll::Pending;
                }
            }
            None => {
                // Buffer is empty, fall through to poll input
            }
        }

        // Buffer is empty, poll the input stream
        match &mut state.input {
            None => {
                // Input exhausted
                Poll::Ready(None)
            }
            Some(input) => match input.as_mut().poll_next(cx) {
                Poll::Ready(Some(Ok(t))) => {
                    // Got an Ok, return it directly
                    Poll::Ready(Some(t))
                }
                Poll::Ready(Some(Err(e))) => {
                    if state.err_dropped {
                        // Err stream is gone, discard this item.
                        // Wake ourselves to continue, but yield to avoid spin loop.
                        cx.waker().wake_by_ref();
                        return Poll::Pending;
                    }
                    // Got an Err, buffer it for the other stream
                    state.buffer = Some(Err(e));
                    // Only clone waker if it's different from current
                    if !state
                        .ok_waker
                        .as_ref()
                        .is_some_and(|w| w.will_wake(cx.waker()))
                    {
                        state.ok_waker = Some(cx.waker().clone());
                    }
                    // Wake the err stream
                    let waker = state.err_waker.take();
                    drop(state); // Release lock before waking
                    if let Some(waker) = waker {
                        waker.wake();
                    }
                    Poll::Pending
                }
                Poll::Ready(None) => {
                    // Input exhausted
                    state.input = None;
                    // Wake err stream so it knows we're done
                    let waker = state.err_waker.take();
                    drop(state); // Release lock before waking
                    if let Some(waker) = waker {
                        waker.wake();
                    }
                    Poll::Ready(None)
                }
                Poll::Pending => {
                    // Input not ready, store our waker
                    // Only clone waker if it's different from current
                    if !state
                        .ok_waker
                        .as_ref()
                        .is_some_and(|w| w.will_wake(cx.waker()))
                    {
                        state.ok_waker = Some(cx.waker().clone());
                    }
                    Poll::Pending
                }
            },
        }
    }
}

impl<S, T, E> FusedStream for OkStream<S, T, E>
where
    S: Stream<Item = Result<T, E>>,
{
    fn is_terminated(&self) -> bool {
        let state = self.shared.lock();
        state.input.is_none() && !matches!(state.buffer, Some(Ok(_)))
    }
}

impl<S, T, E> Stream for ErrStream<S, T, E>
where
    S: Stream<Item = Result<T, E>>,
{
    type Item = E;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut state = self.shared.lock();

        // First check the buffer
        match state.buffer.take() {
            Some(Err(e)) => {
                // It's for us! Wake the other stream so it can poll input.
                let waker = state.ok_waker.take();
                drop(state); // Release lock before waking
                if let Some(waker) = waker {
                    waker.wake();
                }
                return Poll::Ready(Some(e));
            }
            Some(Ok(t)) => {
                // Not for us - check if the other stream was dropped
                if state.ok_dropped {
                    // Ok stream is gone, discard this item and continue polling
                    // input Fall through to input polling
                    // section
                } else {
                    // Put it back and wait for ok stream to consume it
                    state.buffer = Some(Ok(t));
                    // Only clone waker if it's different from current
                    if !state
                        .err_waker
                        .as_ref()
                        .is_some_and(|w| w.will_wake(cx.waker()))
                    {
                        state.err_waker = Some(cx.waker().clone());
                    }
                    // Re-wake the ok stream in case of spurious wakeup
                    let waker = state.ok_waker.take();
                    drop(state); // Release lock before waking
                    if let Some(waker) = waker {
                        waker.wake();
                    }
                    return Poll::Pending;
                }
            }
            None => {
                // Buffer is empty, fall through to poll input
            }
        }

        // Buffer is empty, poll the input stream
        match &mut state.input {
            None => {
                // Input exhausted
                Poll::Ready(None)
            }
            Some(input) => match input.as_mut().poll_next(cx) {
                Poll::Ready(Some(Err(e))) => {
                    // Got an Err, return it directly
                    Poll::Ready(Some(e))
                }
                Poll::Ready(Some(Ok(t))) => {
                    if state.ok_dropped {
                        // Ok stream is gone, discard this item.
                        // Wake ourselves to continue, but yield to avoid spin loop.
                        cx.waker().wake_by_ref();
                        return Poll::Pending;
                    }
                    // Got an Ok, buffer it for the other stream
                    state.buffer = Some(Ok(t));
                    // Only clone waker if it's different from current
                    if !state
                        .err_waker
                        .as_ref()
                        .is_some_and(|w| w.will_wake(cx.waker()))
                    {
                        state.err_waker = Some(cx.waker().clone());
                    }
                    // Wake the ok stream
                    let waker = state.ok_waker.take();
                    drop(state); // Release lock before waking
                    if let Some(waker) = waker {
                        waker.wake();
                    }
                    Poll::Pending
                }
                Poll::Ready(None) => {
                    // Input exhausted
                    state.input = None;
                    // Wake ok stream so it knows we're done
                    let waker = state.ok_waker.take();
                    drop(state); // Release lock before waking
                    if let Some(waker) = waker {
                        waker.wake();
                    }
                    Poll::Ready(None)
                }
                Poll::Pending => {
                    // Input not ready, store our waker
                    // Only clone waker if it's different from current
                    if !state
                        .err_waker
                        .as_ref()
                        .is_some_and(|w| w.will_wake(cx.waker()))
                    {
                        state.err_waker = Some(cx.waker().clone());
                    }
                    Poll::Pending
                }
            },
        }
    }
}

impl<S, T, E> FusedStream for ErrStream<S, T, E>
where
    S: Stream<Item = Result<T, E>>,
{
    fn is_terminated(&self) -> bool {
        let state = self.shared.lock();
        state.input.is_none() && !matches!(state.buffer, Some(Err(_)))
    }
}
