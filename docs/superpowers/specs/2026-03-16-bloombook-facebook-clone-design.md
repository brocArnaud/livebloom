# BloomBook — Vaporwave Facebook Clone via LiveBloom Hot-Swap

## Overview

A Facebook-like social network built entirely through LiveBloom's runtime hot-swap engine. Eight sequential agents progressively build the app live — the user watches a full social network materialize in real-time as compiled Rust modules are swapped in without server restart.

**Style**: Retro/vaporwave aesthetic — deep purple backgrounds, neon pink/cyan accents, glow effects, scanline overlays, pixel art touches.

**Name**: BloomBook (play on Facebook + LiveBloom).

## Architecture

### Approach: Multi-module composition

Each feature is a separate Rust source file compiled into one cdylib. A coordinator module (`content.rs`) imports all sibling modules and composes them into a single HTML page via the `get_html()` FFI export.

```
Agent-compiled sources (in-memory):
  src/content.rs       → get_html() / free_html()  — FFI export, composes all modules
  src/style.rs         → get_style() → String       — vaporwave CSS
  src/navbar.rs        → get_navbar() → String       — top navigation bar
  src/stories.rs       → get_stories() → String      — stories carousel
  src/feed.rs          → get_feed() → String          — posts with reactions
  src/messenger.rs     → get_messenger() → String     — chat sidebar
  src/notifications.rs → get_notifications() → String — dropdown
  src/profiles.rs      → get_profiles() → String      — friend suggestions sidebar
  src/threejs.rs       → get_threejs() → String        — 3D vaporwave bust script
```

Only `content.rs` exports FFI symbols. All other modules are regular Rust functions returning `String`.

### State model

All social data is hardcoded in Rust as static/const data. Users, posts, messages, reactions are generated at compile time, with `rand` for varying counts. No database, no persistence.

### Server-side changes

The Axum server (`main.rs`) serves:
- `GET /` — full HTML shell with HTMX + Three.js + CDN fonts loaded
- `GET /content` — returns composed HTML from hot-swapped module (polled by HTMX every 1s)
- `GET /content?path=/hello` — dynamic routes

The main HTML shell loads external resources (Three.js, HTMX, Google Fonts) and contains the HTMX polling div + Three.js canvas. The `/content` endpoint returns everything else from the compiled module.

## Agent Sequence (8 Phases)

| Phase | Delay | Agent | What it swaps in |
|-------|-------|-------|-----------------|
| 0 | 0s | Initial build | Loading screen — "BloomBook is waking up..." with pulsing animation |
| 1 | 10s | Agent 1 | Base layout + full vaporwave CSS + navbar with logo/search/icons |
| 2 | +8s | Agent 2 | Stories bar — horizontal scroll of 8 circular emoji avatars |
| 3 | +8s | Agent 3 | News feed — 8 posts with varied content, CSS gradient "images" |
| 4 | +8s | Agent 4 | Reactions — emoji row under posts with counts, glow on hover |
| 5 | +8s | Agent 5 | Friend suggestions — right sidebar with 4 suggested users |
| 6 | +8s | Agent 6 | Messenger — floating chat panel, bottom-right, message bubbles |
| 7 | +8s | Agent 7 | Notifications — dropdown from bell icon, 5 notifications, red badge |
| 8 | +8s | Agent 8 | Three.js vaporwave bust in hero area + final CSS polish |

Total demo time: ~74 seconds. Server runs indefinitely after.

## Visual Design

### Color Palette
- Background: `#1a0a2e` → `#16213e` gradient
- Cards: `rgba(255, 255, 255, 0.05)` with `backdrop-filter: blur(10px)`
- Primary accent: hot pink `#ff71ce`
- Secondary accent: cyan `#01cdfe`
- Tertiary: lavender `#b967ff`, mint `#05ffa1`
- Text: light pink `#ffd1dc` (body), white (headings)
- Borders: gradient pink → cyan

### Typography
- Headings: `'Press Start 2P'` (Google Fonts) — pixel art feel
- Body: `'VT323'` (Google Fonts) — retro monospace
- Fallback: system sans-serif

### Effects
- Neon glow: `text-shadow` / `box-shadow` with accent colors
- CRT scanline overlay via CSS `::after` pseudo-element
- Sunset gradient dividers between posts
- Gradient ring borders on story avatars (animated)
- Hover glow on interactive elements

### Layout (Desktop)
```
┌──────────────────────────────────────────────────────┐
│  🌸 BloomBook    [Search...]    🔔(3)  👤 Profile    │  ← Navbar
├──────────────────────────────────────────────────────┤
│        ┌─── Three.js Vaporwave Bust ───┐             │  ← Hero (Phase 8)
│        └───────────────────────────────┘             │
├──────────┬───────────────────────┬───────────────────┤
│ Nav      │ Stories: ○ ○ ○ ○ ○ ○ │  Friend           │
│ 🏠 Feed  ├───────────────────────┤  Suggestions      │
│ 👥 Friends│ Post 1...            │  ┌──────────┐     │
│ 💬 Chat  │ Post 2...            │  │ 👤 User   │     │
│ 🔔 Notif │ Post 3...            │  │ [Add]     │     │
│ 🌸 Bloom │ ...                  │  └──────────┘     │
│          │                      │  ┌──────────┐     │
│          │                      │  │ 👤 User   │     │
│          │                      │  │ [Add]     │     │
│          │                      │  └──────────┘     │
├──────────┴───────────────────────┴───────────────────┤
│                                    ┌─Messenger──────┐│
│                                    │ Chat bubbles   ││
│                                    │ [Type...]      ││
│                                    └────────────────┘│
└──────────────────────────────────────────────────────┘
```

## Feature Specifications

### Users (Static Data)
6 users:
| Handle | Emoji | Status |
|--------|-------|--------|
| AestheticDreamer (you) | 🌸 | "Living in the aesthetic" |
| NeonSunset_99 | 🌅 | "Chasing neon dreams" |
| VaporMike | 🎵 | "Macintosh Plus on repeat" |
| CyberSakura | 🌸 | "Digital cherry blossoms" |
| RetroWave_Luna | 🌙 | "Moonlit synthwave" |
| PixelPhantom | 👾 | "8-bit soul in a 4K world" |

### Posts (8-10 posts)
Mixed content:
- Text posts: vaporwave quotes, aesthetics thoughts, song references
- "Image" posts: CSS gradient art panels (sunset gradients, neon city, palm silhouettes) — pure CSS, no actual images
- Each post has: author avatar+name, timestamp, content, reaction bar
- Timestamps: "Just now", "2h ago", "5h ago", "Yesterday", "2 days ago", etc.

### Reactions
5 types: 👍 Like, ❤️ Love, 😂 Laugh, 😮 Wow, 🌸 Bloom
- Displayed under each post as emoji + count
- Random counts (1-99) per reaction per post
- Active state with neon glow

### Stories Bar
- 8 circular avatars in horizontal scroll container
- Gradient ring border (animated: pink → cyan → lavender → pink)
- First: "Your Story" with + overlay
- Others: user emoji + name below

### Messenger
- Fixed position bottom-right
- Header: "💬 Messenger" + minimize button
- Conversation list: 3 recent chats (user emoji + name + last message preview)
- Open chat: 5 message bubbles alternating left/right
- Input field with placeholder "Type a message..."
- Styled with vaporwave colors, gradient send button

### Notifications
- Bell icon 🔔 in navbar with red badge showing count
- Dropdown panel with 5 notifications:
  - "NeonSunset_99 liked your post" (2m ago)
  - "VaporMike sent you a friend request" (1h ago)
  - "CyberSakura commented: 'A E S T H E T I C'" (3h ago)
  - "RetroWave_Luna shared your post" (Yesterday)
  - "PixelPhantom started following you" (2 days ago)
- Unread items highlighted with left border accent

### Friend Suggestions (Right Sidebar)
4 users with:
- Emoji avatar, display name
- "X mutual friends" text
- "Add Friend" button with gradient border + hover glow

### Three.js Vaporwave Bust
- IcosahedronGeometry(1.5, 1) — gives a faceted, geometric look
- Wireframe MeshBasicMaterial with color `#ff71ce`
- Slow Y-axis rotation (0.005 per frame)
- Canvas placed in hero area between navbar and stories
- Black background matching the dark theme
- Guard against re-initialization (`window._bloomBustLoaded`)

## Testing Strategy

### Unit tests (in lib.rs)
- All module functions return valid HTML strings (non-empty, contains expected elements)
- Manifest generation includes any added dependencies
- Route management (add/get)
- Source file editing
- lib.rs module generation for all source files

### Integration tests
- Full rebuild_and_swap with multi-module source
- Agent sequence produces valid HTML at each phase
- Compilation error handling

## Files Changed
- `src/main.rs` — new HTML shell with CDN resources, 8-agent sequence
- `src/lib.rs` — no structural changes (engine stays the same)
- `backlog.md` — new backlog for BloomBook features
