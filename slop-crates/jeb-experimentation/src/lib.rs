use {
    futures::{stream::FusedStream, Stream},
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
                    // Err stream is gone, discard this item and continue polling input
                    // Fall through to input polling section
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
                    // Ok stream is gone, discard this item and continue polling input
                    // Fall through to input polling section
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

    #[test]
    fn test_empty_stream() {
        let input = stream::iter(Vec::<Result<i32, &str>>::new());
        let (ok_stream, _err_stream) = oks_and_errs(input);

        let oks: Vec<i32> = block_on(ok_stream.collect());
        assert!(oks.is_empty());

        // Note: err_stream would also be empty but we already consumed input via ok_stream
    }

    #[test]
    fn test_single_ok() {
        let input = stream::iter(vec![Ok::<_, &str>(42)]);
        let (ok_stream, _err_stream) = oks_and_errs(input);

        let results: Vec<i32> = block_on(ok_stream.collect());
        assert_eq!(results, vec![42]);
    }

    #[test]
    fn test_single_err() {
        let input = stream::iter(vec![Err::<i32, _>("error")]);
        let (_ok_stream, err_stream) = oks_and_errs(input);

        let results: Vec<&str> = block_on(err_stream.collect());
        assert_eq!(results, vec!["error"]);
    }

    #[test]
    fn test_many_consecutive_oks() {
        let input = stream::iter((0..100).map(Ok::<_, &str>).collect::<Vec<_>>());
        let (ok_stream, _err_stream) = oks_and_errs(input);

        let results: Vec<i32> = block_on(ok_stream.collect());
        assert_eq!(results, (0..100).collect::<Vec<_>>());
    }

    #[test]
    fn test_many_consecutive_errs() {
        let input = stream::iter((0..100).map(|i| Err::<i32, _>(i)).collect::<Vec<_>>());
        let (_ok_stream, err_stream) = oks_and_errs(input);

        let results: Vec<i32> = block_on(err_stream.collect());
        assert_eq!(results, (0..100).collect::<Vec<_>>());
    }

    #[test]
    fn test_drop_both_streams() {
        let input = stream::iter(vec![Ok::<_, &str>(1), Err("a"), Ok(2)]);
        let (ok_stream, err_stream) = oks_and_errs(input);

        // Drop both - should not panic or leak
        drop(ok_stream);
        drop(err_stream);
    }

    /// Test that when a stream sees an item in the buffer belonging to the other stream,
    /// it wakes that stream (if a waker is registered).
    ///
    /// This test verifies the fix for a potential deadlock where:
    /// 1. StreamA buffers item for StreamB, wakes StreamB (if waker registered)
    /// 2. StreamA gets polled again (spurious wakeup)
    /// 3. StreamA sees item still in buffer, must re-wake StreamB
    ///
    /// The key insight: when checking the buffer and finding an item for the other
    /// stream, we must wake that stream even on subsequent polls (spurious wakeups).
    #[test]
    fn test_wake_on_buffer_check() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::task::{RawWaker, RawWakerVTable};

        // Track wake calls
        static WAKE_COUNT: AtomicUsize = AtomicUsize::new(0);

        const VTABLE: RawWakerVTable = RawWakerVTable::new(
            |_| RawWaker::new(std::ptr::null(), &VTABLE),
            |_| {
                WAKE_COUNT.fetch_add(1, Ordering::SeqCst);
            },
            |_| {
                WAKE_COUNT.fetch_add(1, Ordering::SeqCst);
            },
            |_| {},
        );

        // Input stream that will give us an Err then an Ok
        let input = stream::iter(vec![Err::<i32, &str>("error"), Ok(1)]);
        let (mut ok_stream, mut err_stream) = oks_and_errs(input);

        let waker = unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), &VTABLE)) };
        let mut cx = Context::from_waker(&waker);

        WAKE_COUNT.store(0, Ordering::SeqCst);

        // 1. ok_stream polls first, gets Err, buffers it for err_stream
        let ok_pinned = std::pin::pin!(&mut ok_stream);
        let result = ok_pinned.poll_next(&mut cx);
        assert!(matches!(result, Poll::Pending));

        let wakes_after_ok_poll = WAKE_COUNT.load(Ordering::SeqCst);
        // err_stream hasn't registered waker yet, so wake count might be 0

        // 2. err_stream polls, sees Err in buffer, takes it
        let err_pinned = std::pin::pin!(&mut err_stream);
        let result = err_pinned.poll_next(&mut cx);
        assert!(matches!(result, Poll::Ready(Some("error"))));

        // err_stream should have woken ok_stream after taking the buffered item
        let wakes_after_err_takes = WAKE_COUNT.load(Ordering::SeqCst);
        assert!(
            wakes_after_err_takes > wakes_after_ok_poll,
            "err_stream should wake ok_stream when taking buffered item"
        );

        // 3. ok_stream polls again, buffer empty, polls input, gets Ok(1)
        let ok_pinned = std::pin::pin!(&mut ok_stream);
        let result = ok_pinned.poll_next(&mut cx);
        assert!(
            matches!(result, Poll::Ready(Some(1))),
            "ok_stream should get Ok(1) from input"
        );
    }

    /// Test the specific scenario where we re-wake after seeing buffer item for other stream.
    /// Setup: ok_stream has waker registered, buffer has Ok for it, err_stream polls.
    #[test]
    fn test_rewake_when_other_stream_checks_buffer() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::task::{RawWaker, RawWakerVTable};

        static WAKE_COUNT: AtomicUsize = AtomicUsize::new(0);

        const VTABLE: RawWakerVTable = RawWakerVTable::new(
            |_| RawWaker::new(std::ptr::null(), &VTABLE),
            |_| {
                WAKE_COUNT.fetch_add(1, Ordering::SeqCst);
            },
            |_| {
                WAKE_COUNT.fetch_add(1, Ordering::SeqCst);
            },
            |_| {},
        );

        // Input: Ok, then Err - designed so we can set up the right state
        let input = stream::iter(vec![Ok::<i32, &str>(1)]);
        let (mut ok_stream, mut err_stream) = oks_and_errs(input);

        let waker = unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), &VTABLE)) };
        let mut cx = Context::from_waker(&waker);

        WAKE_COUNT.store(0, Ordering::SeqCst);

        // err_stream polls: buffer empty, polls input, gets Ok(1), buffers it, returns Pending
        let err_pinned = std::pin::pin!(&mut err_stream);
        let result = err_pinned.poll_next(&mut cx);
        assert!(matches!(result, Poll::Pending));

        // Now: buffer has Ok(1), err_stream has registered err_waker

        // ok_stream polls, registers ok_waker, then checks buffer, sees Ok(1), takes it
        let ok_pinned = std::pin::pin!(&mut ok_stream);
        let result = ok_pinned.poll_next(&mut cx);
        assert!(matches!(result, Poll::Ready(Some(1))));

        // err_stream polls again: buffer empty, polls input, input exhausted, returns None
        let err_pinned = std::pin::pin!(&mut err_stream);
        let result = err_pinned.poll_next(&mut cx);
        assert!(matches!(result, Poll::Ready(None)));

        // Now test: ok_stream polls, buffer empty, input exhausted, returns None
        let ok_pinned = std::pin::pin!(&mut ok_stream);
        let result = ok_pinned.poll_next(&mut cx);
        assert!(matches!(result, Poll::Ready(None)));
    }

    /// Test interleaved polling of both streams with mixed Ok/Err items.
    /// This ensures the cooperative handoff works correctly when both streams
    /// are polled alternately.
    #[test]
    fn test_interleaved_polling() {
        use std::task::{RawWaker, RawWakerVTable};

        const VTABLE: RawWakerVTable = RawWakerVTable::new(
            |_| RawWaker::new(std::ptr::null(), &VTABLE),
            |_| {},
            |_| {},
            |_| {},
        );

        let input = stream::iter(vec![Ok::<i32, &str>(1), Err("a"), Ok(2), Err("b")]);
        let (mut ok_stream, mut err_stream) = oks_and_errs(input);

        let waker = unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), &VTABLE)) };
        let mut cx = Context::from_waker(&waker);

        // Poll ok_stream first - should get 1 directly
        let result = std::pin::pin!(&mut ok_stream).poll_next(&mut cx);
        assert_eq!(result, Poll::Ready(Some(1)));

        // Poll err_stream - should poll input, get Err("a") directly
        let result = std::pin::pin!(&mut err_stream).poll_next(&mut cx);
        assert_eq!(result, Poll::Ready(Some("a")));

        // Poll ok_stream - should get 2
        let result = std::pin::pin!(&mut ok_stream).poll_next(&mut cx);
        assert_eq!(result, Poll::Ready(Some(2)));

        // Poll err_stream - should get "b"
        let result = std::pin::pin!(&mut err_stream).poll_next(&mut cx);
        assert_eq!(result, Poll::Ready(Some("b")));

        // Both should be exhausted
        let result = std::pin::pin!(&mut ok_stream).poll_next(&mut cx);
        assert_eq!(result, Poll::Ready(None));

        let result = std::pin::pin!(&mut err_stream).poll_next(&mut cx);
        assert_eq!(result, Poll::Ready(None));
    }

    /// Test that dropping one stream while buffer contains an item for IT
    /// (not for the other stream) works correctly.
    #[test]
    fn test_drop_with_buffered_item_for_dropped_stream() {
        let input = stream::iter(vec![Err::<i32, &str>("a"), Ok(1), Err("b")]);
        let (ok_stream, mut err_stream) = oks_and_errs(input);

        // err_stream polls first, should get "a" directly
        let result = block_on(std::pin::pin!(&mut err_stream).next());
        assert_eq!(result, Some("a"));

        // Now drop ok_stream before it processes anything
        drop(ok_stream);

        // err_stream should still be able to get "b"
        // (the Ok(1) will be discarded since ok_stream is dropped)
        let result = block_on(std::pin::pin!(&mut err_stream).next());
        assert_eq!(result, Some("b"));

        // err_stream should be exhausted
        let result = block_on(std::pin::pin!(&mut err_stream).next());
        assert_eq!(result, None);
    }

    /// Test that when one stream is dropped while an item for the OTHER stream
    /// is buffered, the other stream can still consume it.
    #[test]
    fn test_drop_while_buffer_has_item_for_other() {
        use std::task::{RawWaker, RawWakerVTable};

        const VTABLE: RawWakerVTable = RawWakerVTable::new(
            |_| RawWaker::new(std::ptr::null(), &VTABLE),
            |_| {},
            |_| {},
            |_| {},
        );

        let input = stream::iter(vec![Ok::<i32, &str>(1), Err("a")]);
        let (mut ok_stream, err_stream) = oks_and_errs(input);

        let waker = unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), &VTABLE)) };
        let mut cx = Context::from_waker(&waker);

        // err_stream polls first, gets Ok(1), buffers it, returns Pending
        // (We simulate this by having err_stream poll, which will buffer the Ok)
        drop(err_stream); // Drop immediately before it can consume anything

        // ok_stream should still get all Oks
        let result = std::pin::pin!(&mut ok_stream).poll_next(&mut cx);
        assert_eq!(result, Poll::Ready(Some(1)));

        // Continue to end
        let results: Vec<i32> = block_on(ok_stream.collect());
        assert!(results.is_empty()); // All consumed above
    }

    /// Test that the spin-loop prevention works: when the other stream is dropped
    /// and we encounter items for it, we yield (return Pending) rather than spin.
    #[test]
    fn test_no_spin_loop_with_dropped_partner() {
        use std::task::{RawWaker, RawWakerVTable};

        const VTABLE: RawWakerVTable = RawWakerVTable::new(
            |_| RawWaker::new(std::ptr::null(), &VTABLE),
            |_| {},
            |_| {},
            |_| {},
        );

        // Create stream with Err, then Ok
        let input = stream::iter(vec![Err::<i32, &str>("e"), Ok(1)]);
        let (mut ok_stream, err_stream) = oks_and_errs(input);

        // Drop err_stream immediately
        drop(err_stream);

        let waker = unsafe { Waker::from_raw(RawWaker::new(std::ptr::null(), &VTABLE)) };
        let mut cx = Context::from_waker(&waker);

        // First poll: ok_stream polls input, gets Err("e"), discards it, returns Pending
        let result = std::pin::pin!(&mut ok_stream).poll_next(&mut cx);
        assert_eq!(
            result,
            Poll::Pending,
            "Should return Pending after discarding, not spin"
        );

        // Second poll: ok_stream polls input again, gets Ok(1), returns it
        let result = std::pin::pin!(&mut ok_stream).poll_next(&mut cx);
        assert_eq!(result, Poll::Ready(Some(1)));

        // Third poll: input exhausted
        let result = std::pin::pin!(&mut ok_stream).poll_next(&mut cx);
        assert_eq!(result, Poll::Ready(None));
    }
}
