# Aberred Editor

> **Early development.** This project is in its initial phases. Expect breaking changes, missing features, and rough edges.

A map editor built on top of [aberredengine](https://github.com/idaho06/aberredengine), a 2D game engine written in Rust using Bevy ECS and Raylib.

## Prerequisites

This project depends on `aberredengine` as a local path dependency. You must clone it as a sibling directory before building.

```bash
# Clone both repos side by side
git clone https://github.com/idaho06/aberredengine.git
git clone https://github.com/idaho06/aberrededitor.git

# Your directory layout should look like this:
# Projects/
# ├── aberredengine/
# └── aberrededitor/
```

## Building

```bash
cd aberrededitor
cargo build
```

```bash
cargo run          # debug build
cargo run --release  # optimized build
```

## Documentation

- [`docs/architecture.md`](docs/architecture.md) — ECS/ImGui two-layer model, signal bus, scene lifecycle, map data round-trip
- [`docs/patterns.md`](docs/patterns.md) — six recurring design patterns (signal bus, observer dispatch, AppState caches, pending-state dirty encoding, snapshot serialization, MapEntity marker)
- [`docs/gotchas.md`](docs/gotchas.md) — critical constraints: lua feature gate, path relativity, RaylibHandle access, bevy_ecs derive workaround, ImGui texture pointer safety
- [`docs/recipes/`](docs/recipes/) — step-by-step guides: add a component, add a panel, add a menu action, add an asset store

Generated API docs: `cargo doc --no-deps --open`
