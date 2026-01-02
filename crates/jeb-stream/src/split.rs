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
        let waker = {
            let mut state = self.shared.lock();
            state.ok_dropped = true;
            state.err_waker.take()
        };
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
        let waker = {
            let mut state = self.shared.lock();
            state.err_dropped = true;
            state.ok_waker.take()
        };
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
        match state.buffer.take() {
            Some(Ok(t)) => {
                let waker = state.err_waker.take();
                drop(state);
                if let Some(waker) = waker {
                    waker.wake();
                }
                return Poll::Ready(Some(t));
            }
            Some(Err(e)) => {
                if state.err_dropped {
                } else {
                    state.buffer = Some(Err(e));
                    if !state
                        .ok_waker
                        .as_ref()
                        .is_some_and(|w| w.will_wake(cx.waker()))
                    {
                        state.ok_waker = Some(cx.waker().clone());
                    }
                    let waker = state.err_waker.take();
                    drop(state);
                    if let Some(waker) = waker {
                        waker.wake();
                    }
                    return Poll::Pending;
                }
            }
            None => {}
        }
        match &mut state.input {
            None => Poll::Ready(None),
            Some(input) => match input.as_mut().poll_next(cx) {
                Poll::Ready(Some(Ok(t))) => Poll::Ready(Some(t)),
                Poll::Ready(Some(Err(e))) => {
                    if state.err_dropped {
                        cx.waker().wake_by_ref();
                        return Poll::Pending;
                    }
                    state.buffer = Some(Err(e));
                    if !state
                        .ok_waker
                        .as_ref()
                        .is_some_and(|w| w.will_wake(cx.waker()))
                    {
                        state.ok_waker = Some(cx.waker().clone());
                    }
                    let waker = state.err_waker.take();
                    drop(state);
                    if let Some(waker) = waker {
                        waker.wake();
                    }
                    Poll::Pending
                }
                Poll::Ready(None) => {
                    state.input = None;
                    let waker = state.err_waker.take();
                    drop(state);
                    if let Some(waker) = waker {
                        waker.wake();
                    }
                    Poll::Ready(None)
                }
                Poll::Pending => {
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
        match state.buffer.take() {
            Some(Err(e)) => {
                let waker = state.ok_waker.take();
                drop(state);
                if let Some(waker) = waker {
                    waker.wake();
                }
                return Poll::Ready(Some(e));
            }
            Some(Ok(t)) => {
                if state.ok_dropped {
                } else {
                    state.buffer = Some(Ok(t));
                    if !state
                        .err_waker
                        .as_ref()
                        .is_some_and(|w| w.will_wake(cx.waker()))
                    {
                        state.err_waker = Some(cx.waker().clone());
                    }
                    let waker = state.ok_waker.take();
                    drop(state);
                    if let Some(waker) = waker {
                        waker.wake();
                    }
                    return Poll::Pending;
                }
            }
            None => {}
        }
        match &mut state.input {
            None => Poll::Ready(None),
            Some(input) => match input.as_mut().poll_next(cx) {
                Poll::Ready(Some(Err(e))) => Poll::Ready(Some(e)),
                Poll::Ready(Some(Ok(t))) => {
                    if state.ok_dropped {
                        cx.waker().wake_by_ref();
                        return Poll::Pending;
                    }
                    state.buffer = Some(Ok(t));
                    if !state
                        .err_waker
                        .as_ref()
                        .is_some_and(|w| w.will_wake(cx.waker()))
                    {
                        state.err_waker = Some(cx.waker().clone());
                    }
                    let waker = state.ok_waker.take();
                    drop(state);
                    if let Some(waker) = waker {
                        waker.wake();
                    }
                    Poll::Pending
                }
                Poll::Ready(None) => {
                    state.input = None;
                    let waker = state.ok_waker.take();
                    drop(state);
                    if let Some(waker) = waker {
                        waker.wake();
                    }
                    Poll::Ready(None)
                }
                Poll::Pending => {
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
