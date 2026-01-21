# LSP is a Hammer

## The Insight

MCP and CLI can be thought of as LSP clients themselves.

## Current Architecture

```
┌─────────┐     ┌─────────────────┐
│   LSP   │────▶│  DashboardData  │
└─────────┘     └─────────────────┘
                        ▲
┌─────────┐             │
│   MCP   │─────────────┤
└─────────┘             │
                        │
┌─────────┐             │
│   CLI   │─────────────┘
└─────────┘
```

Three separate entry points, each building/accessing `DashboardData` differently.

## Unified Architecture

```
┌─────────────────────────────────────┐
│            Core State               │
│  - DashboardData                    │
│  - VFS (open documents)             │
│  - Config                           │
│  - Rebuild logic                    │
└─────────────────────────────────────┘
          ▲       ▲       ▲
          │       │       │
      ┌───┴───┐ ┌─┴─┐ ┌───┴───┐
      │  LSP  │ │MCP│ │  CLI  │
      │ stdio │ │   │ │       │
      └───────┘ └───┘ └───────┘
```

All three are just transports/interfaces to the same core.

## Benefits

1. **Single source of truth** - VFS overlay logic in one place
2. **Consistent behavior** - all interfaces see the same data
3. **Easier testing** - test the core, transports are thin
4. **Live updates** - CLI could watch for changes like LSP does

## Implementation Ideas

- `TraceyCore` struct owns all state
- Methods like `rebuild()`, `get_diagnostics()`, `find_requirement()`
- LSP/MCP/CLI instantiate `TraceyCore` and call its methods
- For LSP: wrap in `Mutex`, rebuild on `did_change`
- For CLI: one-shot commands, no watching
- For MCP: same as CLI but exposed as tools
