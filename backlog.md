# Backlog — BloomBook (Fully Functional)

## Backend — Data Layer
- [x] ~~B1: Data models (User, Post, Comment, Message, Reaction, FriendRequest, Notification)~~
- [x] ~~B2: In-memory store (SocialStore) with seeded demo data (6 users, 8 posts, messages, notifications)~~
- [x] ~~B3: Store operations: CRUD for posts, comments, reactions, friends, messages, notifications~~

## Backend — API Endpoints
- [x] ~~A1: POST /api/post — create post (text + optional image upload via multipart)~~
- [x] ~~A2: POST /api/post/:id/react/:type — toggle reaction, returns updated reaction bar~~
- [x] ~~A3: POST /api/post/:id/comment — add comment, returns comment HTML fragment~~
- [x] ~~A4: POST /api/friend/:id/request — send friend request, returns "⏳ Pending" button~~
- [x] ~~A5: POST /api/friend/:id/accept — accept friend request, returns "✓ Friends" badge~~
- [x] ~~A6: GET /api/chat/:id — get full conversation HTML with a user~~
- [x] ~~A7: GET /api/notifications — get notifications HTML dropdown~~
- [x] ~~A8: GET /uploads/:filename — serve uploaded media files (tower-http ServeDir)~~
- [x] ~~A9: WS /ws — WebSocket for real-time chat messages~~

## Frontend — Interactive HTMX
- [x] ~~F1: Post creation form with file upload (hx-post, multipart, preview)~~
- [x] ~~F2: Reaction buttons with toggle (hx-post, swap reaction bar, active glow state)~~
- [x] ~~F3: Comment form per post (hx-post, append comment with author info)~~
- [x] ~~F4: Friend request/accept buttons (hx-post, swap button state)~~
- [x] ~~F5: Live chat via WebSocket (JS send/receive, append messages, auto-scroll)~~
- [x] ~~F6: Notification bell click → dropdown (hx-get, toggle visibility)~~
- [x] ~~F7: Dynamic feed rendering from store (not static hot-swap)~~

## Integration
- [x] ~~I1: Hot-swap demo (Phases 0-8) builds the UI shell~~
- [x] ~~I2: After demo, switch to dynamic mode (serve from store, stop HTMX polling)~~
- [x] ~~I3: Media uploads stored on disk (/tmp/bloombook_uploads), served via /uploads/~~
- [x] ~~I4: Visual review + all features tested via Playwright~~
- [x] ~~I5: Clippy clean + 18 tests passing~~

## Security
- [x] ~~S1: HTML escaping on all user-generated content (html_escape function)~~
- [x] ~~S2: Safe DOM manipulation via textContent (no innerHTML with user data)~~
- [x] ~~S3: File upload validates extension, uses UUID filenames~~
