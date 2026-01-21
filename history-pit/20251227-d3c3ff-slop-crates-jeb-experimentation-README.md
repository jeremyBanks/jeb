# jeb-experimentation

**STATUS: EXPERIMENTAL - NOT CONNECTED TO MAIN WORKSPACE**

This crate is an isolated experiment for exploring runtime-agnostic, task-free stream primitives.

## Goal

Implement `oks_and_errs` (and similar stream combinators) that:
- Do NOT spawn tasks (no `tokio::spawn`)
- Do NOT depend on any async runtime
- Are pull-based (only do work when polled)
- Use minimal buffering (single-element handoff)
- Are composable with other futures/streams

## Approach

Manual `Stream` implementation using:
- `std::task::{Context, Poll, Waker}` - core async primitives only
- `Arc<Mutex<SharedState>>` for shared ownership between output streams
- `Pin<Box<S>>` to handle non-Unpin input streams
- Waker-based coordination instead of channels

When one output stream polls and finds an item for the other stream, it:
1. Stores the item in a single-slot buffer
2. Wakes the other stream's waker
3. Returns `Poll::Pending`

This creates cooperative handoff without tasks or unbounded buffering.

## Caveats

- This is exploratory code, not production-ready
- The implementation is more complex than the task-based approach
- Subtle bugs in waker handling are easy to introduce
- This crate is intentionally outside the main workspace
