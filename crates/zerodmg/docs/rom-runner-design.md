# ROM Runner: Unified Script-Based Execution
**Version 1.0 - Design Document**  
*2026-02-14*

## Overview

A unified CLI tool for running Game Boy ROMs with:
- TAS-style input sequences
- Screenshot capture
- Serial I/O testing
- Deterministic, automatable, replayable execution

## Command-Line Interface

```bash
zerodmg run <rom.gb> [OPTIONS]

Options:
  --script <file>      Execute script file
  --serial             Enable stdin/stdout serial I/O (legacy mode)
  --headless           Run without video output
  --timeout <cycles>   Override default timeout
```

## Script Language

**Line-based, simple commands:**

### Timing & Control
```
WAIT <cycles>          # Wait N CPU cycles
WAIT_FRAMES <n>        # Wait N video frames (70224 cycles each)
HALT                   # Stop execution
TIMEOUT <cycles>       # Set max cycles for script (default: 20M)
```

### Input (Joypad)
```
INPUT <buttons>        # Press button(s): A, B, START, SELECT, UP, DOWN, LEFT, RIGHT
INPUT A+B              # Multiple buttons with +
RELEASE <buttons>      # Release button(s)
RELEASE ALL            # Release all buttons
```

### Serial I/O
```
SERIAL_ENABLE          # Start piping stdin → ROM input, ROM output → stdout
SERIAL_DISABLE         # Stop serial piping (ROM can still use serial internally)
SERIAL_WRITE <hex>     # Write specific bytes (e.g., SERIAL_WRITE 0xAB 0xCD)
```

### Capture & Debugging
```
SCREENSHOT <path>      # Save current video frame to PNG
DUMP_STATE <path>      # Save CPU/memory state (JSON)
LOG <message>          # Print message to stderr
```

### Assertions (Future)
```
EXPECT_OUTPUT <string>      # Verify serial output matches
EXPECT_PIXEL <x>,<y> <color> # Check pixel color (#RRGGBB)
EXPECT_REGISTER <reg> <val>  # Check CPU register
```

## Execution Model

### Script Mode
When `--script` is provided:
1. Load ROM
2. Parse script file
3. Execute commands sequentially
4. Exit when script ends or HALT/TIMEOUT reached

**No automatic termination** - script controls execution flow.

### Serial Mode (Legacy)
When `--serial` (no script):
1. Load ROM
2. Enable serial piping
3. Use activity timeout logic (4M cycles, see serial-io-design.md)
4. Exit when timeout or stdin EOF + inactivity

This is equivalent to:
```
SERIAL_ENABLE
TIMEOUT 20000000
```

### Headless Mode
When `--headless`:
- No video rendering (faster execution)
- Screenshots still work (capture internal framebuffer)
- Useful for CI/CD testing

## Example Scripts

### TAS Recording
```bash
# menu-navigation.txt
WAIT_FRAMES 60          # Wait for title screen
INPUT START
WAIT_FRAMES 10
RELEASE START

SCREENSHOT menu.png

INPUT DOWN
WAIT_FRAMES 5
INPUT A
WAIT_FRAMES 30

SCREENSHOT selected.png
HALT
```

### Serial I/O Test
```bash
# test-z85-encode.txt
SERIAL_ENABLE
WAIT 5000000           # Let ROM initialize and process
SERIAL_DISABLE

# Future: EXPECT_OUTPUT "007oA"
HALT
```

Usage:
```bash
echo -ne '\xAB\xCD' | zerodmg run z85-serial.gb --script test-z85-encode.txt
```

### Combined (Input + Serial + Screenshots)
```bash
# game-playthrough.txt
WAIT_FRAMES 120        # Title screen

INPUT START            # Start game
WAIT_FRAMES 30
RELEASE START

SCREENSHOT game-start.png

# Play level 1
INPUT RIGHT
WAIT_FRAMES 60
RELEASE RIGHT

INPUT A                # Jump
WAIT_FRAMES 20
RELEASE A

SCREENSHOT mid-level.png

# Enable debug output
SERIAL_ENABLE
WAIT_FRAMES 60        # Capture debug data
SERIAL_DISABLE

SCREENSHOT level-complete.png
HALT
```

## Implementation Strategy

### Phase 1: Core Infrastructure
1. **Script parser** - Read and tokenize script file
2. **Command executor** - Dispatch to handlers
3. **Basic commands** - WAIT, WAIT_FRAMES, HALT, TIMEOUT, LOG
4. **Script runner binary** - `src/bin/run-rom.rs`

### Phase 2: Input & Capture
5. **INPUT/RELEASE** - Joypad control
6. **SCREENSHOT** - Video frame capture
7. **Example scripts** - TAS samples

### Phase 3: Serial Integration
8. **SERIAL_ENABLE/DISABLE** - Stdin/stdout piping control
9. **SERIAL_WRITE** - Direct byte injection
10. **Refactor serial-io.rs** - Use script runner underneath

### Phase 4: Assertions (Future)
11. **EXPECT_OUTPUT** - Verify serial output
12. **EXPECT_PIXEL** - Visual testing
13. **EXPECT_REGISTER** - CPU state checks

## Design Decisions

### Why line-based instead of structured?
- **Simplicity** - Easy to write by hand, easy to parse
- **Readability** - Clear what's happening at a glance
- **Extensibility** - Easy to add new commands

### Why explicit screenshots instead of auto-capture?
- **Intentionality** - Only capture what matters
- **Storage** - Don't fill disk with thousands of frames
- **Control** - Script author decides when to capture

### Serial timing interaction
**When SERIAL_ENABLE called:**
- Start reading stdin (non-blocking)
- Buffer input into ROM's serial queue
- Pipe ROM output to stdout immediately

**When SERIAL_DISABLE called:**
- Stop reading stdin (discard any buffered)
- Stop outputting ROM serial data
- ROM can still use serial internally (link cable simulation)

**Activity timeout:**
- In script mode: Controlled by TIMEOUT command (default 20M cycles)
- Not based on serial activity - script controls flow
- In legacy --serial mode: Use v2.0 activity timeout (4M cycles)

### Screenshot format
- PNG, 160x144 pixels
- Direct framebuffer capture (no scaling)
- Path can be relative or absolute
- Overwrites if exists

## Error Handling

**Script parse errors:**
```
Error: Unknown command 'WAIY' at line 5
Error: Invalid button 'X' at line 10
Error: Missing argument for WAIT at line 3
```

**Runtime errors:**
```
Error: Timeout reached (20000000 cycles) at line 15
Error: Failed to write screenshot 'test.png': Permission denied
Error: Serial disabled but SERIAL_WRITE called at line 20
```

**Continue on non-fatal errors** (e.g., screenshot fail) with warning.
**Abort on fatal errors** (e.g., ROM crash, timeout).

## Testing Strategy

**Unit tests:**
- Script parser (valid/invalid syntax)
- Command tokenization
- Cycle counting

**Integration tests:**
- Run simple ROM with basic script
- Verify screenshot output
- Verify serial I/O
- Test input sequences

**Example ROMs:**
- `test-roms/input-echo.gb` - Echoes joypad state
- `test-roms/serial-echo.gb` - Echoes serial input
- `test-roms/visual-test.gb` - Shows patterns for screenshot tests

## Future Enhancements

### P1 - Next Version
- **LOOP** construct - Repeat commands N times
- **LABEL/GOTO** - Simple branching
- **DUMP_STATE** - Full CPU/memory snapshot

### P2 - Nice to Have
- **Comments** - `# This is a comment`
- **Variables** - Store/reuse values
- **Conditional execution** - IF/ELSE based on state

### P3 - Advanced
- **Real-time mode** - Run at actual GB speed (not as-fast-as-possible)
- **Breakpoints** - Pause at specific PC/cycles
- **Interactive mode** - REPL for live script execution

## File Organization

```
crates/zerodmg/
├── docs/
│   ├── rom-runner-design.md (this file)
│   └── serial-io-design.md
├── src/
│   └── bin/
│       ├── run-rom.rs (new - script runner)
│       └── serial-io.rs (refactor to use run-rom)
├── examples/
│   └── scripts/
│       ├── basic-test.txt
│       ├── z85-encode-test.txt
│       └── tas-demo.txt
└── test-roms/
    ├── input-echo.gb
    └── serial-echo.gb
```

## Migration Path

**Existing serial-io users:**
```bash
# Old way
echo "data" | cargo run --bin serial-io rom.gb

# New equivalent
echo "data" | cargo run --bin run-rom rom.gb --serial

# Or with script
echo "data" | cargo run --bin run-rom rom.gb --script test.txt
```

**serial-io.rs becomes thin wrapper:**
```rust
// Just call run-rom with --serial flag
fn main() {
    run_rom::main_with_args(&["--serial"]);
}
```

## Open Questions

1. **Frame timing** - Do we need frame-perfect timing or is cycle count enough?
   - **Decision**: Provide both WAIT (cycles) and WAIT_FRAMES (convenience)

2. **Input timing** - How long do buttons stay pressed?
   - **Decision**: Until explicit RELEASE (or RELEASE ALL)

3. **Serial buffering** - How much should we buffer before backpressure?
   - **Decision**: Reuse serial-io-v2.0 logic (10 bytes, configurable via TIMEOUT if needed)

4. **Screenshot on headless** - Does it make sense?
   - **Decision**: Yes - useful for visual regression testing in CI

## Success Criteria

**MVP is successful when:**
- [x] Can execute basic script (WAIT, HALT, LOG)
- [ ] Can control joypad input (INPUT, RELEASE)
- [ ] Can capture screenshots (SCREENSHOT)
- [ ] Can pipe serial I/O (SERIAL_ENABLE/DISABLE)
- [ ] Can replace serial-io.rs functionality
- [ ] Documentation with examples
- [ ] At least 3 example scripts

**v1.0 is production-ready when:**
- [ ] All MVP features working
- [ ] Test coverage >80%
- [ ] CI integration examples
- [ ] Performance acceptable (headless mode fast enough)

---

**Status:** Design complete, ready for implementation  
**Estimated effort:** ~2-3 hours for MVP  
**Risk level:** Low - building on existing serial-io foundation
