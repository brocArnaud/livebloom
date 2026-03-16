mod models;
mod render;
mod store;

use axum::{
    extract::{ws::WebSocket, Multipart, Path, Query, State, WebSocketUpgrade},
    response::Html,
    routing::{get, post},
    Router,
};
use futures::{SinkExt, StreamExt};
use livebloom::LiveBloom;
use models::WsMessage;
use serde::Deserialize;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;
use tower_http::services::ServeDir;
use tracing::{error, info};

use crate::render::html_escape;
use crate::store::SocialStore;

/// Shared application state passed to all handlers
#[derive(Clone)]
struct AppState {
    bloom: LiveBloom,
    store: SocialStore,
    /// When true, /content serves from the live store instead of hot-swap
    dynamic_mode: Arc<std::sync::atomic::AtomicBool>,
}

#[derive(Deserialize)]
struct PathQuery {
    path: Option<String>,
}

#[derive(Deserialize)]
struct ChatQuery {
    user_id: Option<String>,
}

// ═══════════════════════════════════════════════════════════════════
// HOT-SWAP MODULE SOURCES (kept from previous version for demo)
// These are the Rust source strings that get compiled into cdylib.
// After the demo, the app switches to dynamic mode.
// ═══════════════════════════════════════════════════════════════════

// Include the vaporwave CSS module source
const STYLE_SOURCE: &str = include_str!("hotswap/style.rs.txt");
const NAVBAR_SOURCE: &str = include_str!("hotswap/navbar.rs.txt");
const STORIES_SOURCE: &str = include_str!("hotswap/stories.rs.txt");
const FEED_SOURCE: &str = include_str!("hotswap/feed.rs.txt");
// Note: file was extracted from FEED_BASE_SOURCE const
const PROFILES_SOURCE: &str = include_str!("hotswap/profiles.rs.txt");
const MESSENGER_SOURCE: &str = include_str!("hotswap/messenger.rs.txt");
const NOTIFICATIONS_SOURCE: &str = include_str!("hotswap/notifications.rs.txt");
const THREEJS_SOURCE: &str = include_str!("hotswap/threejs.rs.txt");

// Phase content compositors
const CONTENT_LOADING: &str = include_str!("hotswap/content_loading.rs.txt");
const CONTENT_PHASE1: &str = include_str!("hotswap/content_phase1.rs.txt");
const CONTENT_PHASE2: &str = include_str!("hotswap/content_phase2.rs.txt");
const CONTENT_PHASE3: &str = include_str!("hotswap/content_phase3.rs.txt");
const CONTENT_PHASE4: &str = include_str!("hotswap/content_phase4.rs.txt");
const CONTENT_PHASE5: &str = include_str!("hotswap/content_phase5.rs.txt");
const CONTENT_PHASE6: &str = include_str!("hotswap/content_phase6.rs.txt");
const CONTENT_PHASE7: &str = include_str!("hotswap/content_phase7.rs.txt");
const CONTENT_PHASE8: &str = include_str!("hotswap/content_phase8.rs.txt");

// ═══════════════════════════════════════════════════════════════════
// HTML SHELL (served by GET /)
// ═══════════════════════════════════════════════════════════════════

const HTML_SHELL: &str = r##"<!DOCTYPE html>
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
</script>
</body>
</html>"##;

// ═══════════════════════════════════════════════════════════════════
// MAIN
// ═══════════════════════════════════════════════════════════════════

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let bloom = match LiveBloom::new("bloombook") {
        Ok(b) => b,
        Err(e) => {
            error!("Failed to create LiveBloom: {}", e);
            return;
        }
    };

    // Phase 0: loading screen
    bloom.edit_file("src/content.rs", CONTENT_LOADING.to_string());
    if let Err(e) = bloom.rebuild_and_swap("core") {
        error!("Initial build failed: {}", e);
        return;
    }

    let social_store = SocialStore::new_with_seed();

    // Create uploads directory
    let upload_dir = std::path::PathBuf::from("/tmp/bloombook_uploads");
    if let Err(e) = std::fs::create_dir_all(&upload_dir) {
        error!("Failed to create uploads dir: {}", e);
        return;
    }

    let state = AppState {
        bloom: bloom.clone(),
        store: social_store.clone(),
        dynamic_mode: Arc::new(std::sync::atomic::AtomicBool::new(false)),
    };

    let app = Router::new()
        // Page shell
        .route("/", get(|| async { Html(HTML_SHELL) }))
        // Content (hot-swap or dynamic)
        .route("/content", get(handle_content))
        // API endpoints
        .route("/api/post", post(handle_create_post))
        .route("/api/post/:post_id/react/:reaction", post(handle_react))
        .route("/api/post/:post_id/comment", post(handle_comment))
        .route("/api/friend/:user_id/request", post(handle_friend_request))
        .route("/api/friend/:user_id/accept", post(handle_friend_accept))
        .route("/api/chat/:user_id", get(handle_chat))
        .route("/api/chat-list", get(handle_chat_list))
        .route("/api/notifications", get(handle_notifications))
        .route("/api/notifications/read", post(handle_mark_read))
        // WebSocket
        .route("/ws", get(handle_ws_upgrade))
        // Serve uploaded files
        .nest_service("/uploads", ServeDir::new(&upload_dir))
        .with_state(state.clone());

    let listener = match tokio::net::TcpListener::bind("0.0.0.0:3000").await {
        Ok(l) => l,
        Err(e) => {
            error!("Failed to bind to port 3000: {}", e);
            return;
        }
    };
    info!("🌸 BloomBook server started → http://localhost:3000");
    let server = axum::serve(listener, app);

    let server_handle = tokio::spawn(async move {
        if let Err(e) = server.await {
            error!("Server error: {}", e);
        }
    });

    // Run hot-swap demo, then switch to dynamic mode
    run_demo(bloom, state.dynamic_mode.clone()).await;

    info!("🌸 BloomBook is now in DYNAMIC MODE — fully interactive!");
    if let Err(e) = server_handle.await {
        error!("Server task failed: {}", e);
    }
}

// ═══════════════════════════════════════════════════════════════════
// HOT-SWAP DEMO SEQUENCE
// ═══════════════════════════════════════════════════════════════════

async fn run_demo(bloom: LiveBloom, dynamic_mode: Arc<std::sync::atomic::AtomicBool>) {
    #[allow(clippy::type_complexity)]
    let phases: Vec<(&str, Vec<(&str, &str)>, &str)> = vec![
        ("AGENT 1: Base layout + vaporwave CSS + navbar",
         vec![("src/style.rs", STYLE_SOURCE), ("src/navbar.rs", NAVBAR_SOURCE)],
         CONTENT_PHASE1),
        ("AGENT 2: Stories carousel",
         vec![("src/stories.rs", STORIES_SOURCE)],
         CONTENT_PHASE2),
        ("AGENT 3: News feed with posts",
         vec![("src/feed.rs", FEED_SOURCE)],
         CONTENT_PHASE3),
        ("AGENT 4: Reactions on posts",
         vec![],
         CONTENT_PHASE4),
        ("AGENT 5: Friend suggestions + nav sidebar",
         vec![("src/profiles.rs", PROFILES_SOURCE)],
         CONTENT_PHASE5),
        ("AGENT 6: Messenger chat panel",
         vec![("src/messenger.rs", MESSENGER_SOURCE)],
         CONTENT_PHASE6),
        ("AGENT 7: Notifications dropdown",
         vec![("src/notifications.rs", NOTIFICATIONS_SOURCE)],
         CONTENT_PHASE7),
        ("AGENT 8: Three.js vaporwave bust",
         vec![("src/threejs.rs", THREEJS_SOURCE)],
         CONTENT_PHASE8),
    ];

    time::sleep(Duration::from_secs(10)).await;

    for (label, files, content) in phases {
        info!("🌸 {}", label);
        for (path, source) in &files {
            bloom.edit_file(*path, source.to_string());
        }
        bloom.edit_file("src/content.rs", content.to_string());
        if let Err(e) = bloom.rebuild_and_swap_async("core").await {
            error!("{} failed: {}", label, e);
        }
        time::sleep(Duration::from_secs(6)).await;
    }

    // Switch to dynamic mode
    time::sleep(Duration::from_secs(3)).await;
    info!("🌸 AGENT 9: Switching to DYNAMIC mode — all features now interactive!");
    dynamic_mode.store(true, std::sync::atomic::Ordering::Relaxed);
}

// ═══════════════════════════════════════════════════════════════════
// ROUTE HANDLERS
// ═══════════════════════════════════════════════════════════════════

/// GET /content — serves either hot-swap HTML or dynamic content
async fn handle_content(
    State(state): State<AppState>,
    Query(q): Query<PathQuery>,
) -> Html<String> {
    // Check dynamic routes first
    let path = q.path.unwrap_or_default();
    if let Some(html) = state.bloom.get_route(&path) {
        return Html(html);
    }

    // Dynamic mode: serve from store
    if state
        .dynamic_mode
        .load(std::sync::atomic::Ordering::Relaxed)
    {
        let html = render::render_full_page(&state.store).await;
        // Once in dynamic mode, stop HTMX polling (the page is now interactive)
        let html = format!(
            r#"<div hx-swap-oob="true" id="app">{html}</div>
            <script>
            // Stop polling, we're dynamic now
            var app = document.getElementById('app');
            if (app) {{
                app.removeAttribute('hx-get');
                app.removeAttribute('hx-trigger');
                app.removeAttribute('hx-swap');
                htmx.process(app);
            }}
            </script>"#
        );
        return Html(html);
    }

    // Hot-swap mode: serve from compiled module
    Html(state.bloom.get_html())
}

/// POST /api/post — create a new post with optional media
async fn handle_create_post(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Html<String> {
    let current_user_id = state.store.current_user_id().await;
    let mut content = String::new();
    let mut media_url = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "content" => {
                content = field.text().await.unwrap_or_default();
            }
            "media" => {
                let filename = field
                    .file_name()
                    .unwrap_or("upload.bin")
                    .to_string();
                if filename.is_empty() || filename == "upload.bin" {
                    // No file uploaded, skip
                    let _ = field.bytes().await;
                    continue;
                }
                let data = match field.bytes().await {
                    Ok(d) if !d.is_empty() => d,
                    _ => continue,
                };
                // Save to disk
                let ext = std::path::Path::new(&filename)
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("bin");
                let safe_name = format!("{}.{}", uuid::Uuid::new_v4(), ext);
                let path = format!("/tmp/bloombook_uploads/{}", safe_name);
                if let Err(e) = tokio::fs::write(&path, &data).await {
                    error!("Failed to save upload: {}", e);
                    continue;
                }
                media_url = Some(format!("/uploads/{}", safe_name));
            }
            _ => {
                let _ = field.bytes().await;
            }
        }
    }

    if content.trim().is_empty() {
        return Html("<div class='post-card' style='color:red;padding:16px'>Post content cannot be empty</div>".to_string());
    }

    let post = state
        .store
        .create_post(current_user_id, content, media_url)
        .await;
    let user = state.store.get_user(&current_user_id).await.unwrap();

    let post_html = render::render_post(&post, &user, &current_user_id);

    // The HTMX response already adds the post for the current user,
    // so we don't broadcast new_post (would duplicate for the poster).
    Html(post_html)
}

/// POST /api/post/:id/react/:type — toggle a reaction
async fn handle_react(
    State(state): State<AppState>,
    Path((post_id, reaction_type)): Path<(String, String)>,
) -> Html<String> {
    let current_user_id = state.store.current_user_id().await;
    let post_id: uuid::Uuid = match post_id.parse() {
        Ok(id) => id,
        Err(_) => return Html("<div>Invalid post ID</div>".to_string()),
    };

    match state
        .store
        .toggle_reaction(&post_id, &current_user_id, &reaction_type)
        .await
    {
        Some(post) => Html(render::render_reactions(&post, &current_user_id)),
        None => Html("<div>Post not found</div>".to_string()),
    }
}

/// POST /api/post/:id/comment — add a comment
async fn handle_comment(
    State(state): State<AppState>,
    Path(post_id): Path<String>,
    axum::Form(form): axum::Form<CommentForm>,
) -> Html<String> {
    let current_user_id = state.store.current_user_id().await;
    let post_id: uuid::Uuid = match post_id.parse() {
        Ok(id) => id,
        Err(_) => return Html("<div>Invalid post ID</div>".to_string()),
    };

    if form.content.trim().is_empty() {
        return Html(String::new());
    }

    let user = state.store.get_user(&current_user_id).await.unwrap();

    match state
        .store
        .add_comment(&post_id, current_user_id, form.content)
        .await
    {
        Some((comment, _post)) => Html(render::render_comment(&comment, &user)),
        None => Html("<div>Post not found</div>".to_string()),
    }
}

#[derive(Deserialize)]
struct CommentForm {
    content: String,
}

/// POST /api/friend/:id/request
async fn handle_friend_request(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> Html<String> {
    let current_user_id = state.store.current_user_id().await;
    let target_id: uuid::Uuid = match user_id.parse() {
        Ok(id) => id,
        Err(_) => return Html("<div>Invalid user ID</div>".to_string()),
    };

    state
        .store
        .send_friend_request(&current_user_id, &target_id)
        .await;
    Html(format!(
        r#"<span class='friend-status-btn pending' id='friend-btn-{target_id}'>⏳ Pending</span>"#
    ))
}

/// POST /api/friend/:id/accept
async fn handle_friend_accept(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> Html<String> {
    let current_user_id = state.store.current_user_id().await;
    let friend_id: uuid::Uuid = match user_id.parse() {
        Ok(id) => id,
        Err(_) => return Html("<div>Invalid user ID</div>".to_string()),
    };

    state
        .store
        .accept_friend_request(&current_user_id, &friend_id)
        .await;
    Html(r#"<span class='friend-status-btn'>✓ Friends</span>"#.to_string())
}

/// GET /api/chat/:user_id — open a conversation
async fn handle_chat(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> Html<String> {
    let current_user_id = state.store.current_user_id().await;
    let chat_user_id: uuid::Uuid = match user_id.parse() {
        Ok(id) => id,
        Err(_) => return Html("<div>Invalid user ID</div>".to_string()),
    };

    let chat_user = match state.store.get_user(&chat_user_id).await {
        Some(u) => u,
        None => return Html("<div>User not found</div>".to_string()),
    };
    let messages = state
        .store
        .get_conversation(&current_user_id, &chat_user_id)
        .await;
    let chat_list = state.store.get_chat_list(&current_user_id).await;

    Html(render::render_messenger(
        &chat_list,
        Some((&chat_user, &messages)),
        &current_user_id,
    ))
}

/// GET /api/chat-list — show chat list (no active chat)
async fn handle_chat_list(State(state): State<AppState>) -> Html<String> {
    let current_user_id = state.store.current_user_id().await;
    let chat_list = state.store.get_chat_list(&current_user_id).await;
    Html(render::render_messenger(
        &chat_list,
        None,
        &current_user_id,
    ))
}

/// GET /api/notifications
async fn handle_notifications(State(state): State<AppState>) -> Html<String> {
    let current_user_id = state.store.current_user_id().await;
    let notifs = state.store.get_notifications(&current_user_id).await;
    Html(render::render_notifications(&notifs))
}

/// POST /api/notifications/read
async fn handle_mark_read(State(state): State<AppState>) -> Html<String> {
    let current_user_id = state.store.current_user_id().await;
    state.store.mark_notifications_read(&current_user_id).await;
    Html(String::new())
}

// ═══════════════════════════════════════════════════════════════════
// WEBSOCKET
// ═══════════════════════════════════════════════════════════════════

async fn handle_ws_upgrade(
    State(state): State<AppState>,
    Query(q): Query<ChatQuery>,
    ws: WebSocketUpgrade,
) -> axum::response::Response {
    let user_id = q.user_id.unwrap_or_default();
    ws.on_upgrade(move |socket| handle_ws(socket, state, user_id))
}

async fn handle_ws(socket: WebSocket, state: AppState, _user_id: String) {
    let (mut sender, mut receiver) = socket.split();
    let mut broadcast_rx = state.store.broadcast_tx.subscribe();

    // Forward broadcast messages to this client
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = broadcast_rx.recv().await {
            if sender
                .send(axum::extract::ws::Message::Text(msg))
                .await
                .is_err()
            {
                break;
            }
        }
    });

    // Receive messages from this client
    let store = state.store.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(axum::extract::ws::Message::Text(text))) = receiver.next().await {
            if let Ok(WsMessage::Chat {
                from_id,
                to_id,
                content,
            }) = serde_json::from_str::<WsMessage>(&text)
            {
                let from_uuid: uuid::Uuid = match from_id.parse() {
                    Ok(id) => id,
                    Err(_) => continue,
                };
                let to_uuid: uuid::Uuid = match to_id.parse() {
                    Ok(id) => id,
                    Err(_) => continue,
                };
                let safe_content = html_escape(&content);
                store
                    .send_message(from_uuid, to_uuid, safe_content.clone())
                    .await;
                let broadcast = serde_json::to_string(&WsMessage::Chat {
                    from_id: from_id.clone(),
                    to_id: to_id.clone(),
                    content: safe_content,
                })
                .unwrap_or_default();
                let _ = store.broadcast_tx.send(broadcast);
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }
}
