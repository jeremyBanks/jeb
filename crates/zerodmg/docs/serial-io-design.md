# Serial I/O Wrapper Design
**Version 1.0 - Initial Implementation**  
*2026-02-14*

## Problem Statement

We need a wrapper that pipes stdin/stdout through Game Boy serial I/O in a way that:
1. Works reliably with streaming data
2. Runs at realistic timing (so ROMs work on real hardware)
3. Terminates gracefully when processing is complete
4. Doesn't require magic termination protocols

**Current implementation issues:**
- Reads all stdin upfront (blocking, not streaming)
- Fixed 10M cycle timeout (arbitrary)
- Feed rate of 1 byte per 1000 cycles (unrealistic)
- No backpressure handling

## Design Goals

**Primary:** ROM code should work on real hardware without emulator-specific tricks.

**Secondary:** Wrapper should handle various ROM patterns:
- Output-only (generator programs)
- Input-only (sink/validator programs)  
- Streaming processors (filters, encoders)
- Batch processors (read all, process, write all)

## Real Hardware Constraints

**Serial transfer timing:**
- Each byte takes 1024 CPU cycles (~250 µs at 4.19 MHz)
- Transfer triggered by writing 0x81 to SC register
- Interrupt fires when complete
- External device controls when new data arrives

**ROM best practices:**
- Use serial interrupt handler
- HALT when waiting for I/O
- Process data incrementally

## Jeremy's Proposal

**Core idea:** Use cycle-based timeouts to infer stream state without explicit signaling.

**Key points:**
1. ROM assumes stdin exhausted after 4M cycles of inactivity
2. Emulator pauses after 2M cycles ahead of last I/O if stdin has data buffered
3. No magic bytes or special protocols
4. Natural backpressure through timing

**Rationale:** 4M cycles ≈ 1 second of real time. If nothing happens for a full second, stream is probably done.

## Matte's Refinements

**Activity definition:** Track last time either:
- ROM produced output (wrote byte to serial), OR
- We pushed input (new stdin byte available)

**Backpressure:** Instead of cycle limit, use queue size:
- If input queue > 10 bytes, pause emulation
- ROM must consume before we continue
- Prevents unbounded memory growth

**Alternative considered:** Pause if 1M cycles pass without ROM touching serial port (clearer "stuck" signal).

## Final Design (v1.0)

**State tracking:**
```rust
cycles_since_activity: usize = 0
stdin_exhausted: bool = false
input_queue: VecDeque<u8>
output_buffer: Vec<u8>
```

**Activity events:**
- ROM writes serial output → reset counter
- New stdin byte arrives → reset counter, push to queue

**Main loop:**
```
1. Read stdin (non-blocking if available)
2. If stdin EOF: stdin_exhausted = true

3. Run one CPU instruction
4. Increment cycles_since_activity

5. If input_queue.len() > 10:
     Wait until ROM consumes (serial I/O activity)
   
6. If cycles_since_activity > 4_000_000 AND stdin_exhausted:
     Terminate (assume ROM done)

7. If ROM triggered serial output:
     Write to stdout immediately
     Reset cycles_since_activity
```

**Serial transfer simulation:**
- When ROM writes 0x81 to SC:
  - Count 1024 cycles for transfer
  - After 1024 cycles: trigger interrupt, pop next input byte into SB
  - This matches real hardware timing

**Termination conditions:**
- 4M cycles of inactivity + stdin EOF = normal termination
- 20M total cycles (safety limit) = timeout termination

## Trade-offs

**Chosen approach:**
✅ No magic bytes or protocols  
✅ ROMs work on real hardware  
✅ Handles output-only and input-only cases  
✅ Natural backpressure  

**Known limitations:**
⚠️ 4M cycle timeout is arbitrary (but ~1 second is reasonable)  
⚠️ Queue size limit (10 bytes) might need tuning  
⚠️ Doesn't handle ROMs that do heavy processing between I/O (might hit timeout)

**Future improvements:**
- Configurable timeout values
- Better detection of HALT-with-interrupts-disabled (definite end state)
- Wall-clock timeout in addition to cycle count
- Metrics/logging for debugging

## Implementation Notes

**Non-blocking stdin:**
- Use `set_nonblocking(true)` on stdin
- Read in chunks when available
- Buffer internally

**Serial interrupt timing:**
- Track cycles since SC = 0x81
- After 1024 cycles: clear bit 7, trigger interrupt, load next byte
- Matches real hardware behavior

**Queue management:**
- VecDeque for efficient push/pop
- Pause emulation if queue full (backpressure)
- Resume when queue drains

## Testing Strategy

**Test cases:**
1. Echo program (input → output passthrough)
2. Generator (output-only, no input)
3. Sink (input-only, no output)
4. Z85 encoder (4 bytes in → 5 chars out)
5. Large stream (10KB+ data)
6. Slow producer (stdin trickles in)

**Success criteria:**
- No lost bytes
- Correct termination timing
- Stdout matches expected output
- Works with both buffered and streaming stdin

## Open Questions

1. Should backpressure use queue size or cycle threshold?
   - **Decision:** Queue size (simpler, more intuitive)

2. What if ROM legitimately processes for 4M cycles between I/O?
   - **Decision:** Accept this as edge case for v1.0, document as limitation
   - Future: Add configurable timeout or wall-clock fallback

3. Should we simulate realistic serial transfer delay (1024 cycles)?
   - **Decision:** Yes, for hardware compatibility

## Version History

- v1.0 (2026-02-14): Initial design based on Jeremy + Matte discussion
  - Cycle-based activity tracking
  - Queue-based backpressure
  - 4M cycle timeout with stdin EOF
  - 1024-cycle serial transfer simulation

---

**Status:** Ready for implementation  
**Risk level:** Medium (edge cases around timing)  
**Confidence:** This will work for 80%+ of use cases, can iterate based on real usage
