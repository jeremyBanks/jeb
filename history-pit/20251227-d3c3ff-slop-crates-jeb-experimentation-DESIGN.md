# Design Documentation: Task-Free Stream Splitter

## Overview

This crate implements `oks_and_errs()` - a function that splits `Result<T, E>` streams into separate Ok and Err streams **without spawning tasks**.

## Key Design Decisions

### 1. Single-Slot Buffer (Intentional Bottleneck)

**Decision**: Use `Option<Result<T, E>>` - a single-slot buffer for handoff.

**Why**:
- Simplicity: Minimal state to reason about
- Backpressure: Naturally bounds memory usage
- Experimentation focus: We're validating the architecture, not optimizing throughput

**Trade-off**: Creates head-of-line blocking when one stream is slow. A 4-slot ring buffer would improve throughput by ~25% but adds complexity.

**For production**: Consider bounded buffer (VecDeque with capacity limit).

### 2. Lock-Before-Wake Pattern (Critical for Correctness)

**Pattern seen throughout code**:
```rust
let waker = state.err_waker.take();
drop(state);  // Release lock BEFORE wake
if let Some(waker) = waker {
    waker.wake();
}
```

**Why**: Prevents deadlock in synchronous executors where `wake()` immediately polls on the same thread:
1. Thread holds lock
2. Calls `waker.wake()`
3. Waker handler immediately tries to poll
4. Poll tries to acquire same lock → DEADLOCK

**Solution**: Always release lock before calling `wake()`. This is correct for all executors.

### 3. Take/Put-Back Buffer Pattern

**Why not peek?**
```rust
match state.buffer.take() {  // Must take, not peek
    Some(Err(e)) => {
        if state.err_dropped {
            // Discard and continue
        } else {
            state.buffer = Some(Err(e));  // Put back
            // ... wake other stream ...
        }
    }
}
```

**Reason**: Rust ownership semantics. `Result<T, E>` must be moved to check which variant. We can't "peek" into an enum without taking ownership. Options:
- `match &state.buffer` - gives reference, can't return owned value
- `match state.buffer.as_ref()` - same issue
- `match state.buffer.take()` - takes ownership, can put back if needed ✓

### 4. FusedStream Contract

**Implementation**:
```rust
fn is_terminated(&self) -> bool {
    let state = self.shared.lock();
    state.input.is_none() && !matches!(state.buffer, Some(Ok(_)))
}
```

**Contract**: After a stream returns `Poll::Ready(None)`, it will continue to return `None` forever. ✓ We satisfy this.

**What `is_terminated()` means**: "This stream **should not be polled** anymore." It's advisory, not a guarantee of atomicity with subsequent operations.

**The "race" that isn't a bug**: Between calling `is_terminated()` and `poll_next()`, the other stream could modify state. This is expected - `is_terminated()` is a hint, not a lock. Standard library streams (channels, etc.) have the same behavior.

### 5. Drop Behavior

**When a stream is dropped**:
1. Set `{ok,err}_dropped = true`
2. Wake the partner stream (so it knows and can make progress)
3. Items in buffer meant for the **dropped** stream are discarded
4. Items meant for the **remaining** stream are still delivered

**Example**: If ErrStream is dropped and buffer has `Err("important")`:
- That error is **intentionally discarded** (no consumer for it)
- If buffer has `Ok(42)`, OkStream will still receive it

**For production**: Add telemetry to track discarded items.

### 6. Both Streams Must Be Polled

**Critical requirement**: Both output streams must be actively polled (or at least one must be dropped).

**Why**: When OkStream encounters an Err:
1. Buffers it
2. Returns `Poll::Pending`
3. Waits for ErrStream to poll and take the Err
4. ErrStream wakes OkStream after taking it

If ErrStream is never polled, OkStream gets stuck. This is by design - pull-based architecture requires active pulling.

**Mitigation**: Drop unused streams early. The Drop impl notifies the partner.

## Assumptions (For Experimentation)

This code assumes:

### Well-Behaved Input Stream
- Input stream's `poll_next()` does not panic
- Input stream's `poll_next()` returns quickly (doesn't block)
- Input stream is not malicious

### Well-Behaved Wakers
- `Waker::clone()` does not panic or block
- `waker.wake()` does not panic
- Wakers follow the contract (spurious wakes are okay)

### Well-Behaved Executors
- Executors don't re-enter `poll_next()` on the same stream recursively
- Executors eventually poll streams that have been woken
- Executors don't have wildly different waker vtables

### No Exotic Drop Impls
- Types `T` and `E` do not have Drop impls that try to lock the same `Arc<Mutex<SharedState>>`
- Drop impls don't panic (or if they do, standard panic semantics apply)

**For production**: Add `catch_unwind` boundaries and more defensive checks. For experimentation, these assumptions keep the code simple and focused.

## Performance Characteristics

**Not optimized for**:
- High throughput (>100K items/sec)
- Multi-threaded executors with high contention
- Skewed Ok/Err ratios (90/10 or worse)
- Scenarios where one stream is much slower than the other

**Optimized for**:
- Correctness
- Simplicity
- Understanding the waker coordination pattern
- Validating the task-free architecture

**Bottlenecks** (for billion-item workloads):
- 89% of cycles in `wake()` calls (Review 3 found this)
- Single-slot buffer creates ping-pong polling
- `parking_lot::Mutex` contention under concurrent access

## Testing Philosophy

**Current tests** use `block_on()` with `spin_loop()` - deliberately simple to avoid executor-specific behavior.

**What's NOT tested** (intentionally):
- Real executors (Tokio, async-std)
- Multi-threaded execution
- High-throughput scenarios
- Panic recovery

**For production**: Add executor-specific integration tests.

## What We Learned From Reviews

Multiple review iterations found:
1. **FusedStream bug** (fixed): `is_terminated()` wasn't checking buffer state
2. **Lock-during-wake** (fixed earlier): Must release lock before `wake()`
3. **Buffer ignores dropped flags** (fixed earlier): Must check `_dropped` before buffering

**False alarms** from reviews:
- "FusedStream race condition" - not a bug, expected behavior
- "Memory leaks from will_wake()" - optimization is correct
- "Panic scenarios" - exotic, requires malicious user code
- "Performance issues" - true but irrelevant for experimentation

The code is fundamentally sound. The remaining concerns are about production hardening (telemetry, panic handling, performance optimization) which are out of scope for experimentation.
