# Backlog — LiveBloom

## Bugs (Critical)

- [x] ~~B1: Fix buffer overflow in `get_html` — replaced fixed 1024-byte buffer with CString FFI protocol (heap-allocated, no size limit)~~
- [x] ~~B2: Fix file descriptor leak — `OwnedFd` now stored in `LoadedModule` struct, dropped with the module instead of `mem::forget`~~
- [x] ~~B3: Fix HTMX not loaded in main HTML page — added HTMX script tag to inline HTML~~
- [x] ~~B4: Fix race condition on module swap — `get_html` now clones `Arc<Library>` before use, old lib stays alive during execution~~
- [x] ~~B5: Replace all `unwrap()` on mutex locks with poison-safe `lock_or_default` helper~~
- [x] ~~B6: Replace all `unwrap()` in main.rs with proper error handling (match/if-let)~~

## Bugs (Medium)

- [x] ~~B7: Fix Agent 5 dual animation loop — added `cancelAnimationFrame(window._bloomAnimId)` before starting new loop~~
- [x] ~~B8: Fix hardcoded `pub mod content` — `generate_lib_rs()` now dynamically generates mod declarations from sources map~~
- [x] ~~B9: Fix server exits after agent sequence — `server_handle.await` keeps process alive indefinitely~~
- [x] ~~B10: Fix Agent 5 script re-executes every 1s via HTMX polling — added `window._bloomSphereLoaded` guard~~

## Improvements

- [x] ~~I1: Switch from `println!`/`eprintln!` to `tracing` — added `tracing` + `tracing-subscriber` deps, all logs via `info!`/`error!`/`warn!`~~
- [x] ~~I2: Show compilation errors in HTTP response — `last_error` field stores build errors, `fallback_html()` renders them with HTML escaping~~
- [x] ~~I3: Add shared cargo registry cache — `CARGO_HOME` set to persistent `livebloom_cargo_home` dir across rebuilds~~
- [x] ~~I4: Run `cargo build` via `spawn_blocking` — added `rebuild_and_swap_async()` used by all agents in main.rs~~
- [x] ~~I5: Add unit tests — 14 unit tests covering Manifest, routes, edit_file, generate_lib_rs, html_escape, fallback, clone~~
- [x] ~~I6: Add integration tests — 4 integration tests: compile+load, edit+swap, compile error handling, multi-file builds~~
