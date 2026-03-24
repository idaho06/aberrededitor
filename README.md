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
