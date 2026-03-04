# Partli

A particle-life cellular automaton that combines Conway's Game of Life with gravity simulation.

## Overview

Partli simulates particles on a 2D grid where:
- **Conway rules** govern birth and death (3 neighbors to be born, 2-3 to survive)
- **Gravity** pulls particles toward each other
- **Momentum** is conserved through births and deaths
- **Wrapping** creates a toroidal topology

Each live cell contains a particle with continuous position and velocity. The result is an emergent simulation that feels both organic and physical.

## Features

- **Toroidal topology** with optional stagger offset (twisted torus)
- **Momentum conservation** through mathematically proven transfer rules
- **Barnes-Hut approximation** for O(n log n) gravity computation
- **Rate limiting** to control population dynamics
- **Color encoding** based on velocity direction and speed

## Library Usage

```rust
use partli::{GridConfig, Partli, Vec2};

// Create a 10x10 grid
let config = GridConfig::new(10, 10);
let mut sim = Partli::new(config);

// Check initial state
assert_eq!(sim.population(), 0);
assert_eq!(sim.total_momentum(), Vec2::zero());

// Birth a particle at cell (5, 5)
sim.birth_particle(5, 5);
assert_eq!(sim.population(), 1);
```

## Binary Usage

The `gravity` binary renders the simulation to PNG frames:

```bash
cargo run --release -p gravity
```

## Invariants

The implementation guarantees:
- **Momentum conservation**: Total momentum unchanged through births/deaths
- **One particle per cell**: Grid cells are mutually exclusive
- **Wrap idempotency**: Repeated wrapping produces same result

See `src/lib.rs` for mathematical proofs of these invariants.

## Files

- `DESIGN.md` — Complete specification of the simulation
- `partli.jsx` — React/TypeScript web implementation
- `src/lib.rs` — Rust library with property tests
- `src/main.rs` — Video renderer

## Design Document

See `DESIGN.md` for:
- Coordinate system and grid topology
- Tick processing phases (Conway, Gravity, Movement, Color)
- Parameter reference table
- Edge cases and invariants
