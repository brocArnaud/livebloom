use axum::{extract::Query, response::Html, routing::get, Router};
use livebloom::LiveBloom;
use serde::Deserialize;
use std::time::Duration;
use tokio::time;
use tracing::{error, info};

#[derive(Deserialize)]
struct PathQuery {
    path: Option<String>,
}

// ═══════════════════════════════════════════════════════════════════
// MODULE SOURCES — Each const is a Rust source file that gets compiled
// into the hot-swapped cdylib at runtime.
// ═══════════════════════════════════════════════════════════════════

const STYLE_SOURCE: &str = r##"
pub fn get_style() -> String {
    r#"<style>
:root {
    --bg-dark: #1a0a2e;
    --bg-mid: #16213e;
    --card-bg: rgba(255,255,255,0.04);
    --card-border: rgba(255,113,206,0.15);
    --pink: #ff71ce;
    --cyan: #01cdfe;
    --lavender: #b967ff;
    --mint: #05ffa1;
    --text: #ffd1dc;
    --text-dim: rgba(255,209,220,0.5);
    --heading: #ffffff;
}

* { margin:0; padding:0; box-sizing:border-box; }

body {
    background: linear-gradient(135deg, var(--bg-dark) 0%, var(--bg-mid) 100%);
    color: var(--text);
    font-family: 'VT323', monospace;
    min-height: 100vh;
    font-size: 18px;
    overflow-x: hidden;
}

body::after {
    content: '';
    position: fixed;
    top: 0; left: 0;
    width: 100%; height: 100%;
    background: repeating-linear-gradient(0deg, transparent, transparent 2px, rgba(0,0,0,0.03) 2px, rgba(0,0,0,0.03) 4px);
    pointer-events: none;
    z-index: 9999;
}

/* Navbar */
.navbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 24px;
    background: rgba(26,10,46,0.9);
    backdrop-filter: blur(12px);
    border-bottom: 1px solid var(--card-border);
    position: sticky;
    top: 0;
    z-index: 1000;
}
.navbar-logo {
    font-family: 'Press Start 2P', monospace;
    font-size: 14px;
    color: var(--pink);
    text-shadow: 0 0 15px var(--pink), 0 0 30px rgba(185,103,255,0.5);
    text-decoration: none;
}
.navbar-search {
    background: rgba(255,255,255,0.06);
    border: 1px solid rgba(255,113,206,0.2);
    border-radius: 20px;
    padding: 8px 20px;
    color: var(--text);
    font-family: 'VT323', monospace;
    font-size: 16px;
    width: 300px;
    outline: none;
    transition: border-color 0.3s, box-shadow 0.3s;
}
.navbar-search:focus {
    border-color: var(--cyan);
    box-shadow: 0 0 15px rgba(1,205,254,0.3);
}
.navbar-search::placeholder { color: var(--text-dim); }
.navbar-right {
    display: flex;
    align-items: center;
    gap: 20px;
    font-size: 22px;
    position: relative;
}
.navbar-icon {
    position: relative;
    cursor: pointer;
    transition: transform 0.2s;
}
.navbar-icon:hover { transform: scale(1.2); }
.notif-badge {
    position: absolute;
    top: -6px; right: -8px;
    background: linear-gradient(135deg, #ff3860, var(--pink));
    color: white;
    font-size: 11px;
    font-family: 'Press Start 2P', monospace;
    padding: 2px 5px;
    border-radius: 8px;
    min-width: 16px;
    text-align: center;
}

/* Layout */
.app-layout {
    display: flex;
    max-width: 1200px;
    margin: 0 auto;
    gap: 20px;
    padding: 20px;
}
.left-sidebar {
    width: 200px;
    flex-shrink: 0;
    position: sticky;
    top: 70px;
    height: fit-content;
}
.center-col {
    flex: 1;
    min-width: 0;
}
.right-sidebar {
    width: 260px;
    flex-shrink: 0;
    position: sticky;
    top: 70px;
    height: fit-content;
}

/* Cards */
.card {
    background: var(--card-bg);
    border: 1px solid var(--card-border);
    border-radius: 12px;
    padding: 16px;
    margin-bottom: 16px;
    backdrop-filter: blur(8px);
    transition: border-color 0.3s;
}
.card:hover {
    border-color: rgba(1,205,254,0.3);
}

/* Nav links */
.nav-links { list-style: none; }
.nav-links li {
    padding: 10px 14px;
    border-radius: 8px;
    cursor: pointer;
    transition: background 0.2s;
    font-size: 18px;
}
.nav-links li:hover {
    background: rgba(255,113,206,0.1);
}
.nav-links li.active {
    background: rgba(255,113,206,0.15);
    border-left: 3px solid var(--pink);
}

/* Stories */
.stories-bar {
    display: flex;
    gap: 16px;
    overflow-x: auto;
    padding: 16px 0;
    margin-bottom: 16px;
    scrollbar-width: none;
}
.stories-bar::-webkit-scrollbar { display: none; }
.story-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    flex-shrink: 0;
    cursor: pointer;
}
.story-avatar {
    width: 64px; height: 64px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 28px;
    background: rgba(255,255,255,0.05);
    position: relative;
    border: 3px solid transparent;
    background-clip: padding-box;
}
.story-ring {
    position: absolute;
    inset: -4px;
    border-radius: 50%;
    background: linear-gradient(135deg, var(--pink), var(--cyan), var(--lavender), var(--pink));
    background-size: 300% 300%;
    animation: ring-rotate 3s ease infinite;
    z-index: -1;
}
@keyframes ring-rotate {
    0% { background-position: 0% 50%; }
    50% { background-position: 100% 50%; }
    100% { background-position: 0% 50%; }
}
.story-name {
    font-size: 12px;
    color: var(--text-dim);
    max-width: 70px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}
.story-add {
    position: absolute;
    bottom: -2px; right: -2px;
    background: var(--cyan);
    color: var(--bg-dark);
    border-radius: 50%;
    width: 20px; height: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 14px;
    font-weight: bold;
    border: 2px solid var(--bg-dark);
}

/* Posts */
.post-card {
    background: var(--card-bg);
    border: 1px solid var(--card-border);
    border-radius: 12px;
    margin-bottom: 16px;
    backdrop-filter: blur(8px);
    overflow: hidden;
    transition: border-color 0.3s, transform 0.2s;
}
.post-card:hover {
    border-color: rgba(1,205,254,0.25);
    transform: translateY(-1px);
}
.post-header {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 14px 16px 8px;
}
.post-avatar {
    width: 40px; height: 40px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 20px;
    background: rgba(255,255,255,0.06);
    border: 2px solid var(--card-border);
}
.post-author { font-weight: bold; color: var(--heading); font-size: 16px; }
.post-time { font-size: 13px; color: var(--text-dim); }
.post-content { padding: 8px 16px 12px; line-height: 1.5; font-size: 17px; }
.post-image {
    width: 100%;
    height: 200px;
    margin: 0;
}
.post-divider {
    height: 1px;
    background: linear-gradient(90deg, transparent, var(--pink), var(--cyan), transparent);
    margin: 0 16px;
}

/* Reactions */
.reactions-bar {
    display: flex;
    gap: 6px;
    padding: 10px 16px;
    flex-wrap: wrap;
}
.reaction-btn {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 4px 10px;
    border-radius: 16px;
    background: rgba(255,255,255,0.04);
    border: 1px solid rgba(255,255,255,0.08);
    cursor: pointer;
    font-size: 14px;
    transition: all 0.2s;
    color: var(--text-dim);
    font-family: 'VT323', monospace;
}
.reaction-btn:hover {
    background: rgba(255,113,206,0.15);
    border-color: var(--pink);
    box-shadow: 0 0 10px rgba(255,113,206,0.3);
    color: var(--text);
}
.post-stats {
    display: flex;
    justify-content: space-between;
    padding: 6px 16px;
    font-size: 13px;
    color: var(--text-dim);
}

/* Friend suggestions */
.friend-card {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px;
    border-radius: 8px;
    transition: background 0.2s;
    cursor: pointer;
}
.friend-card:hover { background: rgba(255,255,255,0.03); }
.friend-avatar {
    width: 48px; height: 48px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 24px;
    background: rgba(255,255,255,0.06);
}
.friend-info { flex: 1; }
.friend-name { color: var(--heading); font-size: 15px; }
.friend-mutual { font-size: 13px; color: var(--text-dim); }
.add-btn {
    padding: 6px 14px;
    border-radius: 16px;
    border: 1px solid;
    border-image: linear-gradient(135deg, var(--pink), var(--cyan)) 1;
    background: transparent;
    color: var(--cyan);
    font-family: 'VT323', monospace;
    font-size: 15px;
    cursor: pointer;
    transition: all 0.3s;
}
.add-btn:hover {
    background: rgba(1,205,254,0.1);
    box-shadow: 0 0 12px rgba(1,205,254,0.3);
}

/* Messenger */
.messenger-panel {
    position: fixed;
    bottom: 0;
    right: 24px;
    width: 320px;
    background: rgba(26,10,46,0.95);
    backdrop-filter: blur(16px);
    border: 1px solid var(--card-border);
    border-bottom: none;
    border-radius: 12px 12px 0 0;
    z-index: 900;
    display: flex;
    flex-direction: column;
    max-height: 420px;
}
.messenger-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 10px 14px;
    border-bottom: 1px solid var(--card-border);
    font-family: 'Press Start 2P', monospace;
    font-size: 9px;
    color: var(--pink);
}
.messenger-minimize { cursor: pointer; font-size: 18px; }
.chat-list { flex: 1; overflow-y: auto; }
.chat-item {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 14px;
    cursor: pointer;
    transition: background 0.2s;
    border-bottom: 1px solid rgba(255,255,255,0.03);
}
.chat-item:hover { background: rgba(255,113,206,0.08); }
.chat-item-avatar { font-size: 20px; }
.chat-item-info { flex: 1; }
.chat-item-name { font-size: 14px; color: var(--heading); }
.chat-item-preview { font-size: 13px; color: var(--text-dim); overflow:hidden; text-overflow:ellipsis; white-space:nowrap; max-width:180px; }
.chat-messages {
    flex: 1;
    overflow-y: auto;
    padding: 10px;
    display: flex;
    flex-direction: column;
    gap: 8px;
}
.msg-bubble {
    max-width: 75%;
    padding: 8px 12px;
    border-radius: 16px;
    font-size: 14px;
    line-height: 1.4;
}
.msg-them {
    align-self: flex-start;
    background: rgba(255,113,206,0.15);
    border: 1px solid rgba(255,113,206,0.2);
    border-bottom-left-radius: 4px;
}
.msg-you {
    align-self: flex-end;
    background: rgba(1,205,254,0.15);
    border: 1px solid rgba(1,205,254,0.2);
    border-bottom-right-radius: 4px;
}
.chat-input-bar {
    display: flex;
    gap: 8px;
    padding: 10px;
    border-top: 1px solid var(--card-border);
}
.chat-input {
    flex: 1;
    background: rgba(255,255,255,0.06);
    border: 1px solid rgba(255,113,206,0.15);
    border-radius: 16px;
    padding: 8px 14px;
    color: var(--text);
    font-family: 'VT323', monospace;
    font-size: 15px;
    outline: none;
}
.chat-input::placeholder { color: var(--text-dim); }
.chat-send {
    background: linear-gradient(135deg, var(--pink), var(--lavender));
    border: none;
    border-radius: 50%;
    width: 34px; height: 34px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    font-size: 16px;
    transition: box-shadow 0.3s;
}
.chat-send:hover {
    box-shadow: 0 0 15px rgba(255,113,206,0.5);
}

/* Notifications */
.notif-dropdown {
    position: absolute;
    top: 45px;
    right: 0;
    width: 340px;
    background: rgba(26,10,46,0.96);
    backdrop-filter: blur(16px);
    border: 1px solid var(--card-border);
    border-radius: 12px;
    z-index: 1100;
    overflow: hidden;
}
.notif-header {
    padding: 12px 14px;
    font-family: 'Press Start 2P', monospace;
    font-size: 9px;
    color: var(--pink);
    border-bottom: 1px solid var(--card-border);
    display: flex;
    justify-content: space-between;
}
.notif-item {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    padding: 10px 14px;
    border-bottom: 1px solid rgba(255,255,255,0.03);
    transition: background 0.2s;
    cursor: pointer;
}
.notif-item:hover { background: rgba(255,113,206,0.06); }
.notif-item.unread { border-left: 3px solid var(--pink); background: rgba(255,113,206,0.04); }
.notif-avatar { font-size: 20px; flex-shrink: 0; padding-top: 2px; }
.notif-text { font-size: 14px; line-height: 1.4; }
.notif-text strong { color: var(--heading); }
.notif-time { font-size: 12px; color: var(--text-dim); margin-top: 2px; }

/* Three.js hero */
.hero-canvas-wrap {
    display: flex;
    justify-content: center;
    padding: 10px 0;
    margin-bottom: 10px;
}
.hero-canvas-wrap canvas {
    border: 1px solid var(--card-border);
    border-radius: 12px;
}

/* Section headers */
.section-title {
    font-family: 'Press Start 2P', monospace;
    font-size: 10px;
    color: var(--lavender);
    margin-bottom: 12px;
    text-transform: uppercase;
    letter-spacing: 1px;
}

/* Utility */
.glow-text {
    text-shadow: 0 0 10px currentColor;
}
.loading-text {
    text-align: center;
    padding: 40px;
    color: var(--text-dim);
    font-size: 20px;
    animation: pulse 2s ease-in-out infinite;
}
@keyframes pulse { 0%,100%{opacity:1} 50%{opacity:0.4} }
</style>"#.to_string()
}
"##;

const NAVBAR_SOURCE: &str = r##"
pub fn get_navbar(notif_count: usize) -> String {
    let badge = if notif_count > 0 {
        format!(r#"<span class="notif-badge">{}</span>"#, notif_count)
    } else {
        String::new()
    };
    format!(r#"
    <nav class="navbar">
        <a class="navbar-logo" href='#'>🌸 BloomBook</a>
        <input class="navbar-search" type="text" placeholder="🔍 Search the vapor...">
        <div class="navbar-right">
            <span class="navbar-icon" id="notif-bell">🔔{badge}</span>
            <span class="navbar-icon">💬</span>
            <span class="navbar-icon">🌸</span>
        </div>
    </nav>
    "#, badge=badge)
}
"##;

const STORIES_SOURCE: &str = r##"
pub fn get_stories() -> String {
    let users = [
        ("🌸", "Your Story", true),
        ("🌅", "NeonSunset", false),
        ("🎵", "VaporMike", false),
        ("🌸", "CyberSakura", false),
        ("🌙", "Luna", false),
        ("👾", "PixelPhntm", false),
        ("🎮", "RetroGamer", false),
        ("🌊", "WaveRider", false),
    ];
    let mut html = String::from(r#"<div class="stories-bar">"#);
    for (emoji, name, is_you) in users {
        let add = if is_you { r#"<span class="story-add">+</span>"# } else { "" };
        html += &format!(r#"
        <div class="story-item">
            <div class="story-avatar">
                <div class="story-ring"></div>
                {emoji}{add}
            </div>
            <span class="story-name">{name}</span>
        </div>"#);
    }
    html += "</div>";
    html
}
"##;

const FEED_BASE_SOURCE: &str = r##"
pub fn get_feed(with_reactions: bool) -> String {
    let posts = [
        ("🌅", "NeonSunset_99", "2h ago",
         "Just watched the sun dissolve into pixels 🌅",
         Some(("linear-gradient(180deg, #ff6b6b 0%, #ffa07a 40%, #ff71ce 70%, #b967ff 100%)", "Pixel Sunset")),
         [12, 34, 5, 8, 21]),
        ("🎵", "VaporMike", "3h ago",
         "リサフランク420 / 現代のコンピュー 🎵\n\nThis track changed everything. Macintosh Plus forever.",
         None,
         [45, 12, 3, 2, 67]),
        ("🌸", "CyberSakura", "5h ago",
         "Found this abandoned mall. The aesthetics are unreal...",
         Some(("linear-gradient(180deg, #0f0c29 0%, #302b63 50%, #24243e 100%)", "Abandoned Mall")),
         [23, 56, 8, 15, 9]),
        ("🌸", "AestheticDreamer", "6h ago",
         "Late night coding in the v a p o r w a v e ✨\n\nWhen the terminal glows pink, you know it's going to be a good session.",
         None,
         [67, 23, 41, 5, 33]),
        ("🌙", "RetroWave_Luna", "Yesterday",
         "The moon looks different in 16-bit 🌙",
         Some(("linear-gradient(180deg, #0c0032 0%, #190061 30%, #240090 50%, #3500d3 70%, #282828 100%)", "16-bit Moon")),
         [19, 44, 2, 28, 11]),
        ("👾", "PixelPhantom", "Yesterday",
         "New high score on a game that doesn't exist 👾\n\nScore: 999,999,999\nLevel: ∞\nLives: Yes",
         None,
         [88, 7, 62, 3, 15]),
        ("🎵", "VaporMike", "2 days ago",
         "My room at 3am. Pure aesthetic.",
         Some(("linear-gradient(180deg, #12002e 0%, #1a0040 30%, #2d1060 50%, #ff71ce 90%, #ff71ce 100%)", "3AM Vibes")),
         [34, 18, 9, 42, 27]),
        ("🌅", "NeonSunset_99", "3 days ago",
         "Remember when the future was now?\n\nIt still is. We're living in it. Every pixel, every glow, every beat. This is the future our past dreamed of. 🌐",
         None,
         [51, 29, 14, 7, 39]),
    ];

    let reaction_types = ["👍", "❤️", "😂", "😮", "🌸"];

    let mut html = String::new();
    for (emoji, author, time, content, image, counts) in &posts {
        html += r#"<div class="post-card">"#;
        // Header
        html += &format!(r#"<div class="post-header"><div class="post-avatar">{emoji}</div><div><div class="post-author">{author}</div><div class="post-time">{time}</div></div></div>"#);
        // Content
        let content_html = content.replace('\n', "<br>");
        html += &format!(r#"<div class="post-content">{}</div>"#, content_html);
        // Image
        if let Some((gradient, alt)) = image {
            html += &format!(r#"<div class="post-image" style="background:{gradient}" title="{alt}"></div>"#);
        }
        // Divider
        html += r#"<div class="post-divider"></div>"#;
        // Reactions
        if with_reactions {
            html += r#"<div class="reactions-bar">"#;
            for (i, rtype) in reaction_types.iter().enumerate() {
                let count = counts[i];
                if count > 0 {
                    html += &format!(r#"<span class="reaction-btn">{rtype} {count}</span>"#);
                }
            }
            html += r#"</div>"#;
            let total: usize = counts.iter().sum();
            let comment_count = (total % 23) + 1;
            html += &format!(r#"<div class="post-stats"><span>{total} reactions</span><span>{comment_count} comments · {} shares</span></div>"#, (total % 11) + 1);
        }
        html += "</div>";
    }
    html
}
"##;

const PROFILES_SOURCE: &str = r##"
pub fn get_left_sidebar() -> String {
    r#"<aside class="left-sidebar">
        <ul class="nav-links">
            <li class="active">🏠 Feed</li>
            <li>👥 Friends</li>
            <li>💬 Messenger</li>
            <li>🔔 Notifications</li>
            <li>📺 Watch</li>
            <li>🛍️ Marketplace</li>
            <li>🌸 BloomHub</li>
        </ul>
        <div class="card" style="margin-top:16px;text-align:center">
            <div style="font-size:40px;margin-bottom:8px">🌸</div>
            <div style="font-family:'Press Start 2P',monospace;font-size:8px;color:#ff71ce">AestheticDreamer</div>
            <div style="font-size:13px;color:rgba(255,209,220,0.5);margin-top:6px">Living in the aesthetic</div>
        </div>
    </aside>"#.to_string()
}

pub fn get_right_sidebar() -> String {
    let suggestions = [
        ("🎭", "MaskOfVenus", "5 mutual friends"),
        ("🏮", "LanternDrift", "3 mutual friends"),
        ("🦋", "NeonButterfly", "8 mutual friends"),
        ("🎪", "CircusVapor", "2 mutual friends"),
    ];
    let mut html = String::from(r#"<aside class="right-sidebar"><div class="card"><div class="section-title">People You May Know</div>"#);
    for (emoji, name, mutual) in suggestions {
        html += &format!(r#"
        <div class="friend-card">
            <div class="friend-avatar">{emoji}</div>
            <div class="friend-info">
                <div class="friend-name">{name}</div>
                <div class="friend-mutual">{mutual}</div>
            </div>
            <button class="add-btn">+ Add</button>
        </div>"#);
    }
    html += r#"</div>
    <div class="card" style="margin-top:16px">
        <div class="section-title">Trending in Vapor</div>
        <div style="padding:6px 0;border-bottom:1px solid rgba(255,255,255,0.05)">
            <div style="color:#01cdfe;font-size:14px">#VaporwaveIsNotDead</div>
            <div style="font-size:12px;color:rgba(255,209,220,0.4)">4.2K posts</div>
        </div>
        <div style="padding:6px 0;border-bottom:1px solid rgba(255,255,255,0.05)">
            <div style="color:#b967ff;font-size:14px">#AestheticCoding</div>
            <div style="font-size:12px;color:rgba(255,209,220,0.4)">1.8K posts</div>
        </div>
        <div style="padding:6px 0">
            <div style="color:#05ffa1;font-size:14px">#LiveBloom</div>
            <div style="font-size:12px;color:rgba(255,209,220,0.4)">892 posts</div>
        </div>
    </div></aside>"#;
    html
}
"##;

const MESSENGER_SOURCE: &str = r##"
pub fn get_messenger() -> String {
    r#"<div class="messenger-panel">
        <div class="messenger-header">
            <span>💬 MESSENGER</span>
            <span class="messenger-minimize">−</span>
        </div>
        <div class="chat-messages">
            <div class="msg-bubble msg-them">hey, did you see the new vapor collection? 🌸</div>
            <div class="msg-bubble msg-you">omg yes! the sunset series is incredible</div>
            <div class="msg-bubble msg-them">right?? the colors are so surreal</div>
            <div class="msg-bubble msg-you">we should go to that abandoned mall this weekend 📸</div>
            <div class="msg-bubble msg-them">I'm in! let's chase that aesthetic ✨</div>
        </div>
        <div class="chat-input-bar">
            <input class="chat-input" type="text" placeholder="Type a message...">
            <button class="chat-send">📤</button>
        </div>
    </div>"#.to_string()
}
"##;

const NOTIFICATIONS_SOURCE: &str = r##"
pub fn get_notifications() -> String {
    let items = [
        ("🌅", "NeonSunset_99", " liked your post", "2m ago", true),
        ("🎵", "VaporMike", " sent you a friend request", "1h ago", true),
        ("🌸", "CyberSakura", " commented: 'A E S T H E T I C'", "3h ago", true),
        ("🌙", "RetroWave_Luna", " shared your post", "Yesterday", false),
        ("👾", "PixelPhantom", " started following you", "2 days ago", false),
    ];
    let mut html = String::from(r#"<div class="notif-dropdown"><div class="notif-header"><span>NOTIFICATIONS</span><span style="color:#01cdfe;cursor:pointer;font-family:'VT323',monospace;font-size:14px">Mark all read</span></div>"#);
    for (emoji, name, action, time, unread) in items {
        let cls = if unread { "notif-item unread" } else { "notif-item" };
        html += &format!(r#"<div class="{cls}"><span class="notif-avatar">{emoji}</span><div><div class="notif-text"><strong>{name}</strong>{action}</div><div class="notif-time">{time}</div></div></div>"#);
    }
    html += "</div>";
    html
}
"##;

const THREEJS_SOURCE: &str = r##"
pub fn get_threejs() -> String {
    r#"<div class="hero-canvas-wrap"><canvas id="bloom-bust" width="600" height="200"></canvas></div>
    <script>
    if (!window._bloomBustLoaded) {
        window._bloomBustLoaded = true;
        setTimeout(() => {
            const canvas = document.getElementById('bloom-bust');
            if (!canvas) return;
            const w = 600, h = 200;
            const scene = new THREE.Scene();
            const camera = new THREE.PerspectiveCamera(60, w/h, 0.1, 1000);
            const renderer = new THREE.WebGLRenderer({canvas: canvas, alpha: true, antialias: true});
            renderer.setSize(w, h);
            renderer.setClearColor(0x000000, 0);
            camera.position.z = 4;

            // Main icosahedron (bust)
            const geo1 = new THREE.IcosahedronGeometry(1.2, 1);
            const mat1 = new THREE.MeshBasicMaterial({color: 0xff71ce, wireframe: true, transparent: true, opacity: 0.7});
            const bust = new THREE.Mesh(geo1, mat1);
            scene.add(bust);

            // Inner glow sphere
            const geo2 = new THREE.IcosahedronGeometry(0.8, 0);
            const mat2 = new THREE.MeshBasicMaterial({color: 0x01cdfe, wireframe: true, transparent: true, opacity: 0.4});
            const inner = new THREE.Mesh(geo2, mat2);
            scene.add(inner);

            // Outer ring
            const geo3 = new THREE.TorusGeometry(1.8, 0.02, 8, 64);
            const mat3 = new THREE.MeshBasicMaterial({color: 0xb967ff, transparent: true, opacity: 0.5});
            const ring = new THREE.Mesh(geo3, mat3);
            ring.rotation.x = Math.PI / 3;
            scene.add(ring);

            if (window._bloomAnimId) cancelAnimationFrame(window._bloomAnimId);
            function animate() {
                window._bloomAnimId = requestAnimationFrame(animate);
                bust.rotation.y += 0.005;
                bust.rotation.x += 0.002;
                inner.rotation.y -= 0.008;
                inner.rotation.z += 0.003;
                ring.rotation.z += 0.003;
                renderer.render(scene, camera);
            }
            animate();
        }, 500);
    }
    </script>"#.to_string()
}
"##;

// ═══════════════════════════════════════════════════════════════════
// CONTENT.RS SOURCES — The compositor at each phase
// ═══════════════════════════════════════════════════════════════════

const CONTENT_PHASE0: &str = r###"
#[export_name = "get_html"]
pub extern "C" fn get_html() -> *mut std::ffi::c_char {
    let html = r##"
    <div style="display:flex;flex-direction:column;align-items:center;justify-content:center;height:100vh;text-align:center">
        <h1 style="font-family:'Press Start 2P',monospace;font-size:28px;color:#ff71ce;text-shadow:0 0 20px #ff71ce,0 0 40px rgba(185,103,255,0.5);animation:pulse 2s ease-in-out infinite">
            🌸 BloomBook
        </h1>
        <p style="font-family:'VT323',monospace;font-size:24px;color:#01cdfe;margin-top:20px;animation:pulse 2s ease-in-out infinite 0.5s">
            is waking up...
        </p>
        <div style="margin-top:40px;width:200px;height:4px;background:rgba(255,255,255,0.1);border-radius:2px;overflow:hidden">
            <div style="width:100%;height:100%;background:linear-gradient(90deg,#ff71ce,#01cdfe,#b967ff);animation:loading 2s ease-in-out infinite"></div>
        </div>
    </div>
    <style>
        @keyframes pulse{0%,100%{opacity:1}50%{opacity:0.4}}
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
"###;

const CONTENT_PHASE1: &str = r#"
use crate::style;
use crate::navbar;

#[export_name = "get_html"]
pub extern "C" fn get_html() -> *mut std::ffi::c_char {
    let html = format!("{}{}<div class='app-layout'><div class='center-col'><h2 class='loading-text'>Loading feed...</h2></div></div>",
        style::get_style(), navbar::get_navbar(0));
    let c_string = std::ffi::CString::new(html).unwrap_or_default();
    c_string.into_raw()
}

#[export_name = "free_html"]
pub unsafe extern "C" fn free_html(ptr: *mut std::ffi::c_char) {
    if !ptr.is_null() { let _ = std::ffi::CString::from_raw(ptr); }
}
"#;

const CONTENT_PHASE2: &str = r#"
use crate::style;
use crate::navbar;
use crate::stories;

#[export_name = "get_html"]
pub extern "C" fn get_html() -> *mut std::ffi::c_char {
    let html = format!("{}{}<div class='app-layout'><div class='center-col'>{}<h2 class='loading-text'>Loading posts...</h2></div></div>",
        style::get_style(), navbar::get_navbar(0), stories::get_stories());
    let c_string = std::ffi::CString::new(html).unwrap_or_default();
    c_string.into_raw()
}

#[export_name = "free_html"]
pub unsafe extern "C" fn free_html(ptr: *mut std::ffi::c_char) {
    if !ptr.is_null() { let _ = std::ffi::CString::from_raw(ptr); }
}
"#;

const CONTENT_PHASE3: &str = r#"
use crate::style;
use crate::navbar;
use crate::stories;
use crate::feed;

#[export_name = "get_html"]
pub extern "C" fn get_html() -> *mut std::ffi::c_char {
    let html = format!("{}{}<div class='app-layout'><div class='center-col'>{}{}</div></div>",
        style::get_style(), navbar::get_navbar(0), stories::get_stories(), feed::get_feed(false));
    let c_string = std::ffi::CString::new(html).unwrap_or_default();
    c_string.into_raw()
}

#[export_name = "free_html"]
pub unsafe extern "C" fn free_html(ptr: *mut std::ffi::c_char) {
    if !ptr.is_null() { let _ = std::ffi::CString::from_raw(ptr); }
}
"#;

const CONTENT_PHASE4: &str = r#"
use crate::style;
use crate::navbar;
use crate::stories;
use crate::feed;

#[export_name = "get_html"]
pub extern "C" fn get_html() -> *mut std::ffi::c_char {
    let html = format!("{}{}<div class='app-layout'><div class='center-col'>{}{}</div></div>",
        style::get_style(), navbar::get_navbar(0), stories::get_stories(), feed::get_feed(true));
    let c_string = std::ffi::CString::new(html).unwrap_or_default();
    c_string.into_raw()
}

#[export_name = "free_html"]
pub unsafe extern "C" fn free_html(ptr: *mut std::ffi::c_char) {
    if !ptr.is_null() { let _ = std::ffi::CString::from_raw(ptr); }
}
"#;

const CONTENT_PHASE5: &str = r#"
use crate::style;
use crate::navbar;
use crate::stories;
use crate::feed;
use crate::profiles;

#[export_name = "get_html"]
pub extern "C" fn get_html() -> *mut std::ffi::c_char {
    let html = format!("{}{}<div class='app-layout'>{}<div class='center-col'>{}{}</div>{}</div>",
        style::get_style(), navbar::get_navbar(0),
        profiles::get_left_sidebar(),
        stories::get_stories(), feed::get_feed(true),
        profiles::get_right_sidebar());
    let c_string = std::ffi::CString::new(html).unwrap_or_default();
    c_string.into_raw()
}

#[export_name = "free_html"]
pub unsafe extern "C" fn free_html(ptr: *mut std::ffi::c_char) {
    if !ptr.is_null() { let _ = std::ffi::CString::from_raw(ptr); }
}
"#;

const CONTENT_PHASE6: &str = r#"
use crate::style;
use crate::navbar;
use crate::stories;
use crate::feed;
use crate::profiles;
use crate::messenger;

#[export_name = "get_html"]
pub extern "C" fn get_html() -> *mut std::ffi::c_char {
    let html = format!("{}{}<div class='app-layout'>{}<div class='center-col'>{}{}</div>{}</div>{}",
        style::get_style(), navbar::get_navbar(0),
        profiles::get_left_sidebar(),
        stories::get_stories(), feed::get_feed(true),
        profiles::get_right_sidebar(),
        messenger::get_messenger());
    let c_string = std::ffi::CString::new(html).unwrap_or_default();
    c_string.into_raw()
}

#[export_name = "free_html"]
pub unsafe extern "C" fn free_html(ptr: *mut std::ffi::c_char) {
    if !ptr.is_null() { let _ = std::ffi::CString::from_raw(ptr); }
}
"#;

const CONTENT_PHASE7: &str = r#"
use crate::style;
use crate::navbar;
use crate::stories;
use crate::feed;
use crate::profiles;
use crate::messenger;
use crate::notifications;

#[export_name = "get_html"]
pub extern "C" fn get_html() -> *mut std::ffi::c_char {
    let html = format!("{}{}{}<div class='app-layout'>{}<div class='center-col'>{}{}</div>{}</div>{}",
        style::get_style(), navbar::get_navbar(3),
        notifications::get_notifications(),
        profiles::get_left_sidebar(),
        stories::get_stories(), feed::get_feed(true),
        profiles::get_right_sidebar(),
        messenger::get_messenger());
    let c_string = std::ffi::CString::new(html).unwrap_or_default();
    c_string.into_raw()
}

#[export_name = "free_html"]
pub unsafe extern "C" fn free_html(ptr: *mut std::ffi::c_char) {
    if !ptr.is_null() { let _ = std::ffi::CString::from_raw(ptr); }
}
"#;

const CONTENT_PHASE8: &str = r#"
use crate::style;
use crate::navbar;
use crate::stories;
use crate::feed;
use crate::profiles;
use crate::messenger;
use crate::notifications;
use crate::threejs;

#[export_name = "get_html"]
pub extern "C" fn get_html() -> *mut std::ffi::c_char {
    let html = format!("{}{}{}<div class='app-layout'>{}<div class='center-col'>{}{}{}</div>{}</div>{}",
        style::get_style(), navbar::get_navbar(3),
        notifications::get_notifications(),
        profiles::get_left_sidebar(),
        threejs::get_threejs(),
        stories::get_stories(), feed::get_feed(true),
        profiles::get_right_sidebar(),
        messenger::get_messenger());
    let c_string = std::ffi::CString::new(html).unwrap_or_default();
    c_string.into_raw()
}

#[export_name = "free_html"]
pub unsafe extern "C" fn free_html(ptr: *mut std::ffi::c_char) {
    if !ptr.is_null() { let _ = std::ffi::CString::from_raw(ptr); }
}
"#;

// ═══════════════════════════════════════════════════════════════════
// MAIN — Server + Agent sequence
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
    bloom.edit_file("src/content.rs", CONTENT_PHASE0.to_string());
    if let Err(e) = bloom.rebuild_and_swap("core") {
        error!("Initial build failed: {}", e);
        return;
    }

    let bloom_state = bloom.clone();

    let app = Router::new()
        .route(
            "/",
            get(|| async {
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
</script>
</body>
</html>"#)
            }),
        )
        .route(
            "/content",
            get(move |Query(q): Query<PathQuery>| async move {
                let path = q.path.unwrap_or_default();
                if let Some(html) = bloom_state.get_route(&path) {
                    Html(html)
                } else {
                    Html(bloom_state.get_html())
                }
            }),
        );

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

    // === 8 AGENT PHASES ===
    run_agents(bloom).await;

    info!("All agents complete. BloomBook is live!");
    if let Err(e) = server_handle.await {
        error!("Server task failed: {}", e);
    }
}

async fn run_agents(bloom: LiveBloom) {
    // Phase 1: Base layout + CSS + Navbar
    time::sleep(Duration::from_secs(10)).await;
    info!("🌸 AGENT 1: Base layout + vaporwave CSS + navbar");
    bloom.edit_file("src/style.rs", STYLE_SOURCE.to_string());
    bloom.edit_file("src/navbar.rs", NAVBAR_SOURCE.to_string());
    bloom.edit_file("src/content.rs", CONTENT_PHASE1.to_string());
    if let Err(e) = bloom.rebuild_and_swap_async("core").await {
        error!("Agent 1 failed: {}", e);
    }

    // Phase 2: Stories
    time::sleep(Duration::from_secs(8)).await;
    info!("🌸 AGENT 2: Stories carousel");
    bloom.edit_file("src/stories.rs", STORIES_SOURCE.to_string());
    bloom.edit_file("src/content.rs", CONTENT_PHASE2.to_string());
    if let Err(e) = bloom.rebuild_and_swap_async("core").await {
        error!("Agent 2 failed: {}", e);
    }

    // Phase 3: Feed (without reactions)
    time::sleep(Duration::from_secs(8)).await;
    info!("🌸 AGENT 3: News feed with posts");
    bloom.edit_file("src/feed.rs", FEED_BASE_SOURCE.to_string());
    bloom.edit_file("src/content.rs", CONTENT_PHASE3.to_string());
    if let Err(e) = bloom.rebuild_and_swap_async("core").await {
        error!("Agent 3 failed: {}", e);
    }

    // Phase 4: Reactions
    time::sleep(Duration::from_secs(8)).await;
    info!("🌸 AGENT 4: Reactions on posts");
    bloom.edit_file("src/content.rs", CONTENT_PHASE4.to_string());
    if let Err(e) = bloom.rebuild_and_swap_async("core").await {
        error!("Agent 4 failed: {}", e);
    }

    // Phase 5: Sidebars (profiles + nav)
    time::sleep(Duration::from_secs(8)).await;
    info!("🌸 AGENT 5: Friend suggestions + nav sidebar");
    bloom.edit_file("src/profiles.rs", PROFILES_SOURCE.to_string());
    bloom.edit_file("src/content.rs", CONTENT_PHASE5.to_string());
    if let Err(e) = bloom.rebuild_and_swap_async("core").await {
        error!("Agent 5 failed: {}", e);
    }

    // Phase 6: Messenger
    time::sleep(Duration::from_secs(8)).await;
    info!("🌸 AGENT 6: Messenger chat panel");
    bloom.edit_file("src/messenger.rs", MESSENGER_SOURCE.to_string());
    bloom.edit_file("src/content.rs", CONTENT_PHASE6.to_string());
    if let Err(e) = bloom.rebuild_and_swap_async("core").await {
        error!("Agent 6 failed: {}", e);
    }

    // Phase 7: Notifications
    time::sleep(Duration::from_secs(8)).await;
    info!("🌸 AGENT 7: Notifications dropdown");
    bloom.edit_file("src/notifications.rs", NOTIFICATIONS_SOURCE.to_string());
    bloom.edit_file("src/content.rs", CONTENT_PHASE7.to_string());
    if let Err(e) = bloom.rebuild_and_swap_async("core").await {
        error!("Agent 7 failed: {}", e);
    }

    // Phase 8: Three.js + final polish
    time::sleep(Duration::from_secs(8)).await;
    info!("🌸 AGENT 8: Three.js vaporwave bust + final polish");
    bloom.edit_file("src/threejs.rs", THREEJS_SOURCE.to_string());
    bloom.edit_file("src/content.rs", CONTENT_PHASE8.to_string());
    if let Err(e) = bloom.rebuild_and_swap_async("core").await {
        error!("Agent 8 failed: {}", e);
    }
}
