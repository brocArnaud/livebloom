# BloomBook Facebook Clone Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a vaporwave-styled Facebook clone ("BloomBook") that materializes live via 8 LiveBloom hot-swap agent phases.

**Architecture:** Multi-module Rust source files compiled into a single cdylib. Each agent phase adds/replaces source files in LiveBloom's in-memory source map, rebuilds, and swaps. A compositor module (`content.rs`) imports all feature modules and composes the full HTML page. All social data is hardcoded in Rust.

**Tech Stack:** Rust (cdylib via LiveBloom), Axum, HTMX, Three.js, Google Fonts (Press Start 2P, VT323), pure CSS for vaporwave styling.

---

## File Structure

```
src/
├── lib.rs          # LiveBloom engine (NO changes needed)
└── main.rs         # Axum server + HTML shell + 8-agent sequence (REWRITE)
```

All other "files" exist only in LiveBloom's in-memory source map as Rust strings in `main.rs`:
- `src/style.rs` — CSS string
- `src/navbar.rs` — navbar HTML generator
- `src/stories.rs` — stories carousel HTML
- `src/feed.rs` — posts + reactions HTML
- `src/profiles.rs` — friend suggestions sidebar
- `src/messenger.rs` — chat panel HTML
- `src/notifications.rs` — notifications dropdown
- `src/threejs.rs` — Three.js scene script
- `src/content.rs` — FFI compositor (get_html/free_html)

Each agent phase calls `bloom.edit_file(...)` with the Rust source for these modules, then `rebuild_and_swap_async("core")`.

## Chunk 1: Foundation

### Task 1: Update main.rs HTML shell

**Files:**
- Modify: `src/main.rs`

The HTML shell served by `GET /` needs to load all CDN resources and provide the containers for HTMX-polled content + Three.js canvas.

- [ ] **Step 1: Write the new HTML shell in main.rs**

Replace the `GET /` route handler with:

```rust
Html(r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>BloomBook</title>
<script src="https://cdnjs.cloudflare.com/ajax/libs/three.js/r134/three.min.js"></script>
<script src="https://unpkg.com/htmx.org@1.9.10/dist/htmx.min.js"></script>
<link href="https://fonts.googleapis.com/css2?family=Press+Start+2P&family=VT323&display=swap" rel="stylesheet">
<style>
*{margin:0;padding:0;box-sizing:border-box}
body{background:linear-gradient(135deg,#1a0a2e 0%,#16213e 100%);color:#ffd1dc;font-family:'VT323',monospace;min-height:100vh;font-size:18px}
body::after{content:'';position:fixed;top:0;left:0;width:100%;height:100%;background:repeating-linear-gradient(0deg,transparent,transparent 2px,rgba(0,0,0,0.03) 2px,rgba(0,0,0,0.03) 4px);pointer-events:none;z-index:9999}
::-webkit-scrollbar{width:8px}
::-webkit-scrollbar-track{background:#1a0a2e}
::-webkit-scrollbar-thumb{background:linear-gradient(#ff71ce,#01cdfe);border-radius:4px}
</style>
</head>
<body>
<div id="app" hx-get="/content" hx-trigger="every 1s" hx-swap="innerHTML"></div>
<script>
window._bloomAnimId = null;
window._bloomBustLoaded = false;
window._bloomSphereLoaded = false;
</script>
</body>
</html>"#)
```

- [ ] **Step 2: Build and verify shell loads**

```bash
cargo build --release 2>&1 | tail -3
```

- [ ] **Step 3: Commit**

```bash
git add src/main.rs
git commit -m "feat(shell): update HTML shell with CDN resources for BloomBook"
```

### Task 2: Write Phase 0 — Loading screen

**Files:**
- Modify: `src/main.rs`

The initial `content.rs` that LiveBloom starts with shows a loading animation.

- [ ] **Step 1: Write the initial content.rs source**

This is the Rust string for the initial `edit_file("src/content.rs", ...)` in `LiveBloom::new()` or the initial agent phase. It returns a centered "BloomBook is waking up..." with a pulsing CSS animation.

```rust
bloom.edit_file("src/content.rs", r#"
#[export_name = "get_html"]
pub extern "C" fn get_html() -> *mut std::ffi::c_char {
    let html = r##"
    <div style="display:flex;flex-direction:column;align-items:center;justify-content:center;height:100vh;text-align:center">
        <h1 style="font-family:'Press Start 2P',monospace;font-size:28px;color:#ff71ce;text-shadow:0 0 20px #ff71ce,0 0 40px #b967ff;animation:pulse 2s ease-in-out infinite">
            🌸 BloomBook
        </h1>
        <p style="font-family:'VT323',monospace;font-size:24px;color:#01cdfe;margin-top:20px;animation:pulse 2s ease-in-out infinite 0.5s">
            is waking up...
        </p>
        <div style="margin-top:40px;width:200px;height:4px;background:#1a0a2e;border-radius:2px;overflow:hidden">
            <div style="width:100%;height:100%;background:linear-gradient(90deg,#ff71ce,#01cdfe,#b967ff);animation:loading 2s ease-in-out infinite"></div>
        </div>
    </div>
    <style>
        @keyframes pulse{0%,100%{opacity:1}50%{opacity:0.5}}
        @keyframes loading{0%{transform:translateX(-100%)}50%{transform:translateX(0%)}100%{transform:translateX(100%)}}
    </style>
    "##;
    let c_string = std::ffi::CString::new(html).unwrap_or_default();
    c_string.into_raw()
}

#[export_name = "free_html"]
pub unsafe extern "C" fn free_html(ptr: *mut std::ffi::c_char) {
    if !ptr.is_null() { let _ = std::ffi::CString::from_raw(ptr); }
}
"#.to_string());
```

- [ ] **Step 2: Build and test**

```bash
cargo build --release 2>&1 | tail -3
```

- [ ] **Step 3: Commit**

```bash
git add src/main.rs
git commit -m "feat(phase0): add BloomBook loading screen"
```

### Task 3: Write Phase 1 — Base layout + vaporwave CSS + navbar

**Files:**
- Modify: `src/main.rs`

Agent 1 swaps in two files: `style.rs` (full vaporwave CSS) and `navbar.rs` (top bar), plus updates `content.rs` to compose them.

- [ ] **Step 1: Write style.rs source string**

Full vaporwave CSS as a Rust function returning a `<style>` tag string. Includes:
- Layout grid (left sidebar, center content, right sidebar)
- Card styles with glassmorphism
- Navbar styles
- Button styles with gradient borders
- Neon glow utilities
- Story avatar ring animation
- Post card styles
- Messenger panel styles
- Notification dropdown styles
- Scrollbar customization
- All vaporwave color variables

- [ ] **Step 2: Write navbar.rs source string**

Returns the top navbar HTML:
- Left: 🌸 BloomBook logo (Press Start 2P font, pink glow)
- Center: search input with glass styling
- Right: notification bell 🔔 with badge, profile link 👤

- [ ] **Step 3: Write content.rs compositor for Phase 1**

```rust
pub mod style;
pub mod navbar;

#[export_name = "get_html"]
pub extern "C" fn get_html() -> *mut std::ffi::c_char {
    let html = format!("{}<div class='app-layout'>{}<main class='main-content'>\
        <div class='center-col'><h2 class='loading-text'>Loading feed...</h2></div>\
        </main></div>",
        style::get_style(), navbar::get_navbar());
    let c_string = std::ffi::CString::new(html).unwrap_or_default();
    c_string.into_raw()
}
```

- [ ] **Step 4: Wire Agent 1 in run_agents()**

```rust
time::sleep(Duration::from_secs(10)).await;
info!("AGENT 1: Base layout + vaporwave CSS + navbar");
bloom.edit_file("src/style.rs", STYLE_SOURCE.to_string());
bloom.edit_file("src/navbar.rs", NAVBAR_SOURCE.to_string());
bloom.edit_file("src/content.rs", CONTENT_PHASE1.to_string());
bloom.rebuild_and_swap_async("core").await?;
```

- [ ] **Step 5: Build and test**
- [ ] **Step 6: Commit**

```bash
git commit -m "feat(phase1): add base layout, vaporwave CSS, navbar"
```

## Chunk 2: Social Features (Phases 2-4)

### Task 4: Write Phase 2 — Stories bar

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Write stories.rs source string**

Returns a horizontal scrolling row of 8 story circles:
- "Your Story" first (🌸 + plus icon)
- 7 friends with emoji avatars
- Animated gradient ring borders (pink→cyan→lavender)
- Names below each circle
- Horizontal scroll with overflow-x

- [ ] **Step 2: Update content.rs to include stories**

Add `pub mod stories;` and insert `stories::get_stories()` in the center column above the feed area.

- [ ] **Step 3: Wire Agent 2**
- [ ] **Step 4: Build and test**
- [ ] **Step 5: Commit**

```bash
git commit -m "feat(phase2): add stories carousel"
```

### Task 5: Write Phase 3 — News feed with posts

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Write feed.rs source string**

Generates 8 posts. Each post struct built inline:
- Author (emoji + name), timestamp
- Content: text or text + CSS gradient art panel
- CSS gradient "images": sunset (`linear-gradient(#ff6b6b, #ffa07a, #ff71ce)`), neon city, palm silhouette, etc.
- Post card with glassmorphism styling
- Placeholder for reactions (added in Phase 4)

Post content examples:
1. NeonSunset_99: "Just watched the sun dissolve into pixels 🌅" + sunset gradient panel
2. VaporMike: "リサフランク420 / 現代のコンピュー 🎵" (text only)
3. CyberSakura: "Found this aesthetic mall abandoned..." + neon city gradient
4. You (AestheticDreamer): "Late night coding in the v a p o r w a v e" (text only)
5. RetroWave_Luna: "The moon looks different in 16-bit 🌙" + moonrise gradient
6. PixelPhantom: "New high score on a game that doesn't exist 👾" (text only)
7. VaporMike: "My room at 3am" + purple room gradient
8. NeonSunset_99: "Remember when the future was now? It still is." (text only)

- [ ] **Step 2: Update content.rs to include feed**
- [ ] **Step 3: Wire Agent 3**
- [ ] **Step 4: Build and test**
- [ ] **Step 5: Commit**

```bash
git commit -m "feat(phase3): add news feed with 8 posts"
```

### Task 6: Write Phase 4 — Reactions

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Update feed.rs to include reactions**

Add reaction bar below each post:
- 5 emoji reactions: 👍 Like, ❤️ Love, 😂 Laugh, 😮 Wow, 🌸 Bloom
- Random counts (1-99) per reaction per post (use `rand` if available, or deterministic based on post index)
- Styled as row of pill-shaped buttons with emoji + count
- Hover: neon glow effect
- Comment count and share count below reactions

Since we might not want to add `rand` dep to keep builds fast, use deterministic pseudo-random based on post index:
```rust
fn pseudo_count(post: usize, reaction: usize) -> usize {
    ((post * 17 + reaction * 31 + 7) % 89) + 1
}
```

- [ ] **Step 2: Update content.rs**
- [ ] **Step 3: Wire Agent 4**
- [ ] **Step 4: Build and test**
- [ ] **Step 5: Commit**

```bash
git commit -m "feat(phase4): add reactions to posts"
```

## Chunk 3: Social Panels (Phases 5-7)

### Task 7: Write Phase 5 — Friend suggestions sidebar

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Write profiles.rs source string**

Right sidebar with:
- "People You May Know" header
- 4 friend suggestion cards:
  - Emoji avatar (large)
  - Display name
  - "X mutual friends" text
  - "Add Friend" button with gradient border + hover glow
- Left navigation sidebar:
  - 🏠 Feed, 👥 Friends, 💬 Messenger, 🔔 Notifications, 🌸 Bloom links
  - Styled as vertical list with hover highlights

- [ ] **Step 2: Update content.rs to include left sidebar + right sidebar**
- [ ] **Step 3: Wire Agent 5**
- [ ] **Step 4: Build and test**
- [ ] **Step 5: Commit**

```bash
git commit -m "feat(phase5): add friend suggestions and nav sidebar"
```

### Task 8: Write Phase 6 — Messenger

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Write messenger.rs source string**

Fixed-position bottom-right chat panel:
- Header: "💬 Messenger" + minimize button (styled, non-functional)
- Conversation list: 3 chats
  - NeonSunset_99: "see you in the vapor..."
  - CyberSakura: "check this aesthetic 🌸"
  - VaporMike: "new track dropped"
- One open conversation (NeonSunset_99) with 5 messages:
  - Alternating left (them) / right (you) bubbles
  - Them: pink-tinted bubbles, You: cyan-tinted bubbles
  - Content: casual vaporwave chat
- Input bar: text input + gradient send button 📤
- All non-functional (static HTML) but fully styled

- [ ] **Step 2: Update content.rs to include messenger**
- [ ] **Step 3: Wire Agent 6**
- [ ] **Step 4: Build and test**
- [ ] **Step 5: Commit**

```bash
git commit -m "feat(phase6): add messenger chat panel"
```

### Task 9: Write Phase 7 — Notifications

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Write notifications.rs source string**

Notification dropdown (shown as always-open for demo):
- Positioned below the bell icon in navbar
- 5 notification items:
  - "NeonSunset_99 liked your post" — 2m ago
  - "VaporMike sent you a friend request" — 1h ago
  - "CyberSakura commented: 'A E S T H E T I C'" — 3h ago
  - "RetroWave_Luna shared your post" — Yesterday
  - "PixelPhantom started following you" — 2 days ago
- Unread items: left border accent (pink), slightly brighter background
- Each with emoji avatar, text, timestamp
- "Mark all as read" link at bottom

- [ ] **Step 2: Update navbar.rs to show notification count badge**
- [ ] **Step 3: Update content.rs**
- [ ] **Step 4: Wire Agent 7**
- [ ] **Step 5: Build and test**
- [ ] **Step 6: Commit**

```bash
git commit -m "feat(phase7): add notifications dropdown"
```

## Chunk 4: Three.js + Polish (Phase 8) & Finalization

### Task 10: Write Phase 8 — Three.js vaporwave bust + final polish

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Write threejs.rs source string**

Returns a `<script>` tag that:
- Creates a canvas in the hero area (between navbar and stories)
- Builds an IcosahedronGeometry(1.5, 1) — faceted geometric look
- Wireframe MeshBasicMaterial with color `#ff71ce`
- Slow Y rotation (0.005/frame)
- Dark transparent background matching page
- Guard: `if (!window._bloomBustLoaded)`
- Responsive sizing to container

- [ ] **Step 2: Update content.rs for final layout with hero canvas**
- [ ] **Step 3: Final CSS polish in style.rs**

Add any final touches:
- Smooth transitions on content appearance
- Proper z-indexing for messenger/notifications
- Mobile-friendly max-widths

- [ ] **Step 4: Wire Agent 8**
- [ ] **Step 5: Build and test**
- [ ] **Step 6: Commit**

```bash
git commit -m "feat(phase8): add Three.js vaporwave bust and final polish"
```

### Task 11: Update backlog.md

**Files:**
- Modify: `backlog.md`

- [ ] **Step 1: Write new backlog with all BloomBook items**
- [ ] **Step 2: Commit**

```bash
git commit -m "docs: update backlog for BloomBook"
```

### Task 12: Visual review loop

- [ ] **Step 1: Start server**

```bash
cargo run --release &
```

- [ ] **Step 2: Wait for full agent sequence (~75s)**
- [ ] **Step 3: Take screenshot with Playwright at each phase**
- [ ] **Step 4: Identify and fix visual issues**
- [ ] **Step 5: Repeat until visual quality is excellent**
- [ ] **Step 6: Final commit**

```bash
git commit -m "fix(ui): visual polish from review loop"
```

### Task 13: Final tests + cleanup

- [ ] **Step 1: Run cargo test**

```bash
cargo test
```

All 18 existing tests should still pass. Add tests if new public functions were added to lib.rs.

- [ ] **Step 2: Run cargo clippy**

```bash
cargo clippy -- -D warnings
```

- [ ] **Step 3: Final commit + push**

```bash
git push origin test/facebook-clone
```
