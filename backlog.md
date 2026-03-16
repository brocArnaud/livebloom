# Backlog — LiveBloom

## v0.1.0 — Engine Hardening (COMPLETED)

- [x] ~~B1: Fix buffer overflow in `get_html`~~
- [x] ~~B2: Fix file descriptor leak~~
- [x] ~~B3: Fix HTMX not loaded in main page~~
- [x] ~~B4: Fix race condition on module swap~~
- [x] ~~B5: Replace unwrap() on mutex locks~~
- [x] ~~B6: Replace unwrap() in main.rs~~
- [x] ~~B7: Fix Agent 5 dual animation loop~~
- [x] ~~B8: Fix hardcoded pub mod content~~
- [x] ~~B9: Fix server exits after agent sequence~~
- [x] ~~B10: Fix Agent 5 script re-execution spam~~
- [x] ~~I1: Switch to tracing~~
- [x] ~~I2: Show compilation errors in HTTP response~~
- [x] ~~I3: Shared cargo registry cache~~
- [x] ~~I4: Run cargo build via spawn_blocking~~
- [x] ~~I5+I6: Unit + integration tests (18 tests)~~

## BloomBook — Vaporwave Facebook Clone (COMPLETED)

### Phase 0: Loading Screen
- [x] ~~Pulsing "BloomBook is waking up..." with gradient loading bar~~

### Phase 1: Base Layout + CSS + Navbar
- [x] ~~Full vaporwave CSS (glassmorphism, neon glows, scanline overlay, gradient scrollbar)~~
- [x] ~~Navbar with neon pink logo, glass search bar, icon buttons~~
- [x] ~~3-column responsive layout (left sidebar, center, right sidebar)~~

### Phase 2: Stories Carousel
- [x] ~~8 circular story avatars with animated gradient ring borders~~
- [x] ~~"Your Story" with + icon~~
- [x] ~~Horizontal scroll~~

### Phase 3: News Feed
- [x] ~~8 posts with varied content (text, Japanese, emoji, multi-line)~~
- [x] ~~4 CSS gradient "image" posts (sunset, abandoned mall, moonrise, 3AM vibes)~~
- [x] ~~Post cards with glassmorphism hover effects~~

### Phase 4: Reactions
- [x] ~~5 reaction types: 👍 Like, ❤️ Love, 😂 Laugh, 😮 Wow, 🌸 Bloom~~
- [x] ~~Deterministic counts per post~~
- [x] ~~Reaction bar + stats (total reactions, comments, shares)~~
- [x] ~~Hover glow effect on reaction buttons~~

### Phase 5: Sidebars
- [x] ~~Left nav sidebar (Feed, Friends, Messenger, Notifications, Watch, Marketplace, BloomHub)~~
- [x] ~~User profile card (AestheticDreamer)~~
- [x] ~~Right sidebar: 4 friend suggestions with "+ Add" buttons~~
- [x] ~~Trending hashtags (#VaporwaveIsNotDead, #AestheticCoding, #LiveBloom)~~

### Phase 6: Messenger
- [x] ~~Fixed bottom-right chat panel~~
- [x] ~~5 alternating chat bubbles (pink for them, cyan for you)~~
- [x] ~~Input bar with gradient send button~~
- [x] ~~Glassmorphism panel styling~~

### Phase 7: Notifications
- [x] ~~Dropdown with 5 notifications (liked, friend request, comment, share, follow)~~
- [x] ~~Unread highlight with pink left border~~
- [x] ~~Red badge on bell icon (count: 3)~~
- [x] ~~"Mark all read" action link~~

### Phase 8: Three.js + Final Polish
- [x] ~~Three.js vaporwave bust (IcosahedronGeometry wireframe, pink + cyan inner + lavender ring)~~
- [x] ~~Hero canvas with transparent background~~
- [x] ~~Animation guard (window._bloomBustLoaded)~~
- [x] ~~Notification dropdown positioning fix~~

### Verification
- [x] ~~cargo clippy -- zero warnings~~
- [x] ~~cargo test -- 18/18 passing~~
- [x] ~~Visual review via Playwright -- all phases verified~~
