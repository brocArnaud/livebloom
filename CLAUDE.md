# CLAUDE.md — LiveBloom

## What is LiveBloom?

A Rust **runtime hot-swap engine** that recompiles Rust code and dynamically reloads shared libraries (`.so`) **in memory** via Linux `memfd_create`, without restarting the server. Agents can modify source code, trigger recompilation, and swap loaded native modules while an Axum HTTP server continues serving requests.

## Stack

- **Language**: Rust (stable, Linux-only)
- **HTTP**: Axum + Tokio
- **Hot-swap**: `libloading` + `nix` (`memfd_create`)
- **Frontend**: Inline HTML + HTMX (polling) + Three.js (3D)
- **Build**: Cargo (single crate)

## Architecture

```
src/
├── lib.rs      # LiveBloom engine (sources, manifest, routes, module loading)
└── main.rs     # Axum server + 5-agent demo sequence
```

Single flat crate. `LiveBloom` struct holds all state behind `Arc<Mutex<_>>` for thread-safe cloning across Axum handlers and agent tasks.

### Hot-swap flow

1. Agent calls `edit_file()` → updates in-memory source map
2. Agent calls `rebuild_and_swap()` →
   - Writes sources + generated `Cargo.toml` to a `TempDir`
   - Runs `cargo build --release` → produces `cdylib` `.so`
   - Reads `.so` bytes → writes to `memfd_create` anonymous FD
   - Loads via `libloading::Library::new("/proc/self/fd/<N>")`
   - Stores in `loaded_modules` map, replacing previous module
3. Next `get_html()` call uses the new module's `get_html` C symbol

## Principles

- **Linux-only**: relies on `memfd_create`, `/proc/self/fd/`, `.so` — no cross-platform fallback
- **KISS**: simplest solution that works
- **Safety**: no `unwrap()` on external data, proper error propagation with `anyhow`
- **Logging**: use `tracing`, not `println!`

## Conventions

- Rust: `rustfmt` + `clippy` clean (zero warnings)
- Errors: propagate with `?` and `anyhow::Result`, no `unwrap()` in library code
- Naming: `snake_case`
- Commit messages: `feat(scope): description` / `fix(scope): description`

## Dev

```bash
cargo build                              # build
cargo test                               # all tests
cargo clippy -- -D warnings              # lint
cargo run                                # run server on 0.0.0.0:3000
```

Always bind to `0.0.0.0` for dev servers (container-friendly).

## Compact instructions

Preserve: test output (failures only), code diffs, architecture decisions, current task progress.
Discard: passing test details, verbose build output, file listings.
