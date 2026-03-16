use crate::models::*;
use crate::store::*;
use chrono::Utc;
use uuid::Uuid;

/// Escape HTML special characters to prevent XSS
pub fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Format a timestamp as relative time
pub fn relative_time(dt: &chrono::DateTime<Utc>) -> String {
    let now = Utc::now();
    let diff = now.signed_duration_since(*dt);
    let mins = diff.num_minutes();
    if mins < 1 {
        "Just now".into()
    } else if mins < 60 {
        format!("{}m ago", mins)
    } else if mins < 1440 {
        format!("{}h ago", mins / 60)
    } else if mins < 2880 {
        "Yesterday".into()
    } else {
        format!("{} days ago", mins / 1440)
    }
}

/// Render a single post card
pub fn render_post(post: &Post, author: &User, current_user_id: &Uuid) -> String {
    let pid = post.id;
    let time = relative_time(&post.created_at);
    let content_html = html_escape(&post.content).replace('\n', "<br>");

    let mut html = format!(
        r#"<div class='post-card' id='post-{pid}'>
        <div class='post-header'>
            <div class='post-avatar'>{emoji}</div>
            <div><div class='post-author'>{name}</div><div class='post-time'>{time}</div></div>
        </div>
        <div class='post-content'>{content}</div>"#,
        emoji = html_escape(&author.emoji),
        name = html_escape(&author.username),
        content = content_html
    );

    // Media — media_url is server-controlled (from /uploads/ path), safe to use in src
    if let Some(url) = &post.media_url {
        html += &format!(
            r#"<div class='post-image'><img src='{url}' style='width:100%;max-height:400px;object-fit:cover;border-radius:0'></div>"#,
            url = html_escape(url)
        );
    } else if let Some(grad) = &post.css_gradient {
        html += &format!(r#"<div class='post-image' style='background:{grad};height:200px'></div>"#,
            grad = html_escape(grad));
    }

    html += r#"<div class='post-divider'></div>"#;

    // Reactions bar
    html += &render_reactions(post, current_user_id);

    // Stats
    let total = post.total_reactions();
    let comments = post.comments.len();
    html += &format!(
        r#"<div class='post-stats'><span>{total} reactions</span><span>{comments} comments</span></div>"#
    );

    // Comments section
    html += &format!(r#"<div class='comments-section' id='comments-{pid}'>"#);
    for comment in &post.comments {
        html += &render_comment_inline(comment);
    }
    html += "</div>";

    // Comment form
    html += &format!(
        r#"<form class='comment-form' hx-post='/api/post/{pid}/comment' hx-target='#comments-{pid}' hx-swap='beforeend' hx-on::after-request="this.reset()">
        <input type='text' name='content' placeholder='Write a comment...' class='comment-input' required autocomplete='off'>
        <button type='submit' class='comment-submit'>→</button>
        </form>"#
    );

    html += "</div>";
    html
}

fn render_comment_inline(comment: &Comment) -> String {
    let time = relative_time(&comment.created_at);
    format!(
        r#"<div class='comment-item' id='comment-{}'>
        <div class='comment-text'>{}</div>
        <div class='comment-time'>{}</div>
        </div>"#,
        comment.id,
        html_escape(&comment.content),
        time
    )
}

pub fn render_comment(comment: &Comment, author: &User) -> String {
    let time = relative_time(&comment.created_at);
    format!(
        r#"<div class='comment-item' id='comment-{}'>
        <span class='comment-author-emoji'>{}</span>
        <span class='comment-author-name'>{}</span>
        <span class='comment-text'>{}</span>
        <span class='comment-time'>{}</span>
        </div>"#,
        comment.id,
        html_escape(&author.emoji),
        html_escape(&author.username),
        html_escape(&comment.content),
        time
    )
}

pub fn render_reactions(post: &Post, current_user_id: &Uuid) -> String {
    let pid = post.id;
    let mut html = format!(r#"<div class='reactions-bar' id='reactions-{pid}'>"#);
    for (rtype, emoji) in REACTION_TYPES {
        let count = post.reaction_count(rtype);
        let active = post.user_reacted(current_user_id, rtype);
        let active_class = if active { " reaction-active" } else { "" };
        html += &format!(
            r#"<span class='reaction-btn{active_class}' hx-post='/api/post/{pid}/react/{rtype}' hx-target='#reactions-{pid}' hx-swap='outerHTML'>{emoji} {count}</span>"#
        );
    }
    html += "</div>";
    html
}

/// Render the full feed
pub fn render_feed(posts: &[Post], users: &std::collections::HashMap<Uuid, User>, current_user_id: &Uuid) -> String {
    let mut html = String::new();
    for post in posts {
        if let Some(author) = users.get(&post.author_id) {
            html += &render_post(post, author, current_user_id);
        }
    }
    html
}

/// Render post creation form
pub fn render_post_form() -> String {
    r#"<div class='card post-form-card'>
        <form hx-post='/api/post' hx-target='#feed' hx-swap='afterbegin' hx-encoding='multipart/form-data' hx-on::after-request="this.reset();this.querySelector('.upload-preview').textContent=''">
            <textarea name='content' class='post-textarea' placeholder="What's on your mind? ✨" required rows='3'></textarea>
            <div class='post-form-actions'>
                <label class='upload-label'>
                    📷 Photo/Video
                    <input type='file' name='media' accept='image/*,video/*' style='display:none'>
                </label>
                <button type='submit' class='post-submit-btn'>Post 🌸</button>
            </div>
            <div class='upload-preview'></div>
        </form>
    </div>"#
        .to_string()
}

/// Render friend suggestion card
pub fn render_friend_card(
    user: &User,
    status: &FriendStatus,
    mutual_count: usize,
) -> String {
    let uid = user.id;
    let btn = match status {
        FriendStatus::Friends => {
            r#"<span class='friend-status-btn'>✓ Friends</span>"#.to_string()
        }
        FriendStatus::RequestSent => {
            format!(r#"<span class='friend-status-btn pending' id='friend-btn-{uid}'>⏳ Pending</span>"#)
        }
        FriendStatus::RequestReceived => {
            format!(
                r#"<span id='friend-btn-{uid}'><button class='add-btn' hx-post='/api/friend/{uid}/accept' hx-target='#friend-btn-{uid}' hx-swap='outerHTML'>✓ Accept</button></span>"#
            )
        }
        FriendStatus::None => {
            format!(
                r#"<span id='friend-btn-{uid}'><button class='add-btn' hx-post='/api/friend/{uid}/request' hx-target='#friend-btn-{uid}' hx-swap='outerHTML'>+ Add</button></span>"#
            )
        }
    };

    format!(
        r#"<div class='friend-card'>
        <div class='friend-avatar'>{emoji}</div>
        <div class='friend-info'>
            <div class='friend-name'>{name}</div>
            <div class='friend-mutual'>{mutual} mutual friends</div>
        </div>
        {btn}
        </div>"#,
        emoji = html_escape(&user.emoji),
        name = html_escape(&user.username),
        mutual = mutual_count
    )
}

/// Render notifications dropdown content
pub fn render_notifications(notifs: &[(Notification, User)]) -> String {
    let mut html = String::from(
        r#"<div class='notif-header'><span>NOTIFICATIONS</span>
        <span style='color:#01cdfe;cursor:pointer;font-family:VT323,monospace;font-size:14px' hx-post='/api/notifications/read' hx-swap='none'>Mark all read</span></div>"#,
    );
    if notifs.is_empty() {
        html += r#"<div class='notif-item'><div class='notif-text' style='color:rgba(255,209,220,0.5)'>No notifications yet</div></div>"#;
    }
    for (notif, from_user) in notifs {
        let cls = if notif.read {
            "notif-item"
        } else {
            "notif-item unread"
        };
        let time = relative_time(&notif.created_at);
        let action = notif.kind.action_text();
        html += &format!(
            r#"<div class='{cls}'>
            <span class='notif-avatar'>{emoji}</span>
            <div><div class='notif-text'><strong>{name}</strong> {action}</div>
            <div class='notif-time'>{time}</div></div></div>"#,
            emoji = html_escape(&from_user.emoji),
            name = html_escape(&from_user.username)
        );
    }
    html
}

/// Render messenger panel
pub fn render_messenger(
    chat_list: &[(User, Option<Message>)],
    active_chat: Option<(&User, &[Message])>,
    current_user_id: &Uuid,
) -> String {
    let mut html = String::from(
        r#"<div class='messenger-panel' id='messenger-panel'>
        <div class='messenger-header'>
            <span>💬 MESSENGER</span>
            <span class='messenger-minimize' onclick='this.closest(".messenger-panel").classList.toggle("minimized")'>−</span>
        </div>"#,
    );

    if let Some((chat_user, messages)) = active_chat {
        let chat_uid = chat_user.id;
        html += &format!(
            r#"<div class='chat-active-header' hx-get='/api/chat-list' hx-target='#messenger-panel' hx-swap='outerHTML'>
            ← {emoji} {name}</div>"#,
            emoji = html_escape(&chat_user.emoji),
            name = html_escape(&chat_user.username)
        );
        html += &format!(r#"<div class='chat-messages' id='chat-messages-{chat_uid}'>"#);
        for msg in messages {
            let cls = if msg.from_id == *current_user_id {
                "msg-bubble msg-you"
            } else {
                "msg-bubble msg-them"
            };
            html += &format!(r#"<div class='{cls}'>{}</div>"#, html_escape(&msg.content));
        }
        html += "</div>";
        html += &format!(
            r#"<div class='chat-input-bar'>
            <input class='chat-input' type='text' id='chat-input-{chat_uid}' placeholder='Type a message...' autocomplete='off'
                onkeydown="if(event.key==='Enter'){{sendChat('{chat_uid}',this.value);this.value=''}}">
            <button class='chat-send' onclick="sendChat('{chat_uid}',document.getElementById('chat-input-{chat_uid}').value);document.getElementById('chat-input-{chat_uid}').value=''">📤</button>
            </div>"#
        );
    } else {
        html += r#"<div class='chat-list'>"#;
        for (user, last_msg) in chat_list {
            let uid = user.id;
            let preview = last_msg
                .as_ref()
                .map(|m| {
                    let mut s = m.content.clone();
                    s.truncate(30);
                    html_escape(&s)
                })
                .unwrap_or_default();
            html += &format!(
                r#"<div class='chat-item' hx-get='/api/chat/{uid}' hx-target='#messenger-panel' hx-swap='outerHTML'>
                <span class='chat-item-avatar'>{emoji}</span>
                <div class='chat-item-info'>
                    <div class='chat-item-name'>{name}</div>
                    <div class='chat-item-preview'>{preview}</div>
                </div></div>"#,
                emoji = html_escape(&user.emoji),
                name = html_escape(&user.username)
            );
        }
        html += r#"</div>"#;
    }

    html += "</div>";
    html
}

/// Render the full dynamic page content
pub async fn render_full_page(store: &SocialStore) -> String {
    let current_user_id = store.current_user_id().await;
    let current_user = store.get_user(&current_user_id).await.unwrap();
    let posts = store.get_posts().await;
    let all_users = store.get_all_users().await;
    let users_map: std::collections::HashMap<Uuid, User> =
        all_users.iter().map(|u| (u.id, u.clone())).collect();
    let non_friends = store.get_non_friends(&current_user_id).await;
    let chat_list = store.get_chat_list(&current_user_id).await;
    let unread = store.unread_count(&current_user_id).await;

    let mut html = String::new();

    // CSS
    html += &get_dynamic_style();

    // Navbar
    let badge = if unread > 0 {
        format!(r#"<span class='notif-badge'>{unread}</span>"#)
    } else {
        String::new()
    };
    html += &format!(
        r#"<nav class='navbar'>
        <a class='navbar-logo' href='/'>🌸 BloomBook</a>
        <input class='navbar-search' type='text' placeholder='🔍 Search the vapor...'>
        <div class='navbar-right'>
            <span class='navbar-icon' hx-get='/api/notifications' hx-target='#notif-dropdown' hx-swap='innerHTML' hx-trigger='click' id='notif-bell'>🔔{badge}</span>
            <span class='navbar-icon'>💬</span>
            <span class='navbar-icon'>🌸</span>
            <div class='notif-dropdown' id='notif-dropdown' style='display:none'></div>
        </div></nav>"#
    );

    // Layout
    html += "<div class='app-layout'>";

    // Left sidebar
    html += &format!(
        r#"<aside class='left-sidebar'>
        <ul class='nav-links'>
            <li class='active'>🏠 Feed</li>
            <li>👥 Friends</li>
            <li>💬 Messenger</li>
            <li>🔔 Notifications</li>
            <li>📺 Watch</li>
            <li>🛍️ Marketplace</li>
            <li>🌸 BloomHub</li>
        </ul>
        <div class='card' style='margin-top:16px;text-align:center'>
            <div style='font-size:40px;margin-bottom:8px'>{emoji}</div>
            <div style='font-family:Press Start 2P,monospace;font-size:8px;color:#ff71ce'>{name}</div>
            <div style='font-size:13px;color:rgba(255,209,220,0.5);margin-top:6px'>{status}</div>
        </div></aside>"#,
        emoji = html_escape(&current_user.emoji),
        name = html_escape(&current_user.username),
        status = html_escape(&current_user.status)
    );

    // Center column
    html += "<div class='center-col'>";

    // Three.js hero
    html += r#"<div class='hero-canvas-wrap'><canvas id='bloom-bust' width='600' height='200'></canvas></div>"#;
    html += &get_threejs_script();

    // Stories
    html += &render_stories(&all_users);

    // Post form
    html += &render_post_form();

    // Feed
    html += "<div id='feed'>";
    html += &render_feed(&posts, &users_map, &current_user_id);
    html += "</div>";

    html += "</div>"; // center-col

    // Right sidebar
    html += "<aside class='right-sidebar'>";
    html += "<div class='card'><div class='section-title'>People You May Know</div>";
    for user in non_friends.iter().take(4) {
        let status = store
            .get_friend_status(&current_user_id, &user.id)
            .await;
        let mutual = (user.friends.len() % 6) + 1;
        html += &render_friend_card(user, &status, mutual);
    }
    html += "</div>";

    // Trending
    html += r#"<div class='card' style='margin-top:16px'>
        <div class='section-title'>Trending in Vapor</div>
        <div style='padding:6px 0;border-bottom:1px solid rgba(255,255,255,0.05)'>
            <div style='color:#01cdfe;font-size:14px'>#VaporwaveIsNotDead</div>
            <div style='font-size:12px;color:rgba(255,209,220,0.4)'>4.2K posts</div>
        </div>
        <div style='padding:6px 0;border-bottom:1px solid rgba(255,255,255,0.05)'>
            <div style='color:#b967ff;font-size:14px'>#AestheticCoding</div>
            <div style='font-size:12px;color:rgba(255,209,220,0.4)'>1.8K posts</div>
        </div>
        <div style='padding:6px 0'>
            <div style='color:#05ffa1;font-size:14px'>#LiveBloom</div>
            <div style='font-size:12px;color:rgba(255,209,220,0.4)'>892 posts</div>
        </div></div>"#;
    html += "</aside>";
    html += "</div>"; // app-layout

    // Messenger
    html += &render_messenger(&chat_list, None, &current_user_id);

    // WebSocket + notification toggle script
    html += &get_client_scripts(&current_user_id.to_string());

    html
}

fn render_stories(users: &[User]) -> String {
    let mut html = String::from(r#"<div class='stories-bar'>"#);
    for (i, user) in users.iter().enumerate() {
        let add = if i == 0 {
            r#"<span class='story-add'>+</span>"#
        } else {
            ""
        };
        let name = if i == 0 { "Your Story" } else { &user.username };
        html += &format!(
            r#"<div class='story-item'>
            <div class='story-avatar'><div class='story-ring'></div>{}{add}</div>
            <span class='story-name'>{name}</span></div>"#,
            html_escape(&user.emoji),
            name = html_escape(name)
        );
    }
    html += "</div>";
    html
}

fn get_threejs_script() -> String {
    r#"<script>
    if (!window._bloomBustLoaded) {
        window._bloomBustLoaded = true;
        setTimeout(function() {
            var canvas = document.getElementById('bloom-bust');
            if (!canvas || typeof THREE === 'undefined') return;
            var w = 600, h = 200;
            var scene = new THREE.Scene();
            var camera = new THREE.PerspectiveCamera(60, w/h, 0.1, 1000);
            var renderer = new THREE.WebGLRenderer({canvas: canvas, alpha: true, antialias: true});
            renderer.setSize(w, h);
            renderer.setClearColor(0x000000, 0);
            camera.position.z = 4;
            var bust = new THREE.Mesh(new THREE.IcosahedronGeometry(1.2, 1), new THREE.MeshBasicMaterial({color: 0xff71ce, wireframe: true, transparent: true, opacity: 0.7}));
            scene.add(bust);
            var inner = new THREE.Mesh(new THREE.IcosahedronGeometry(0.8, 0), new THREE.MeshBasicMaterial({color: 0x01cdfe, wireframe: true, transparent: true, opacity: 0.4}));
            scene.add(inner);
            var ring = new THREE.Mesh(new THREE.TorusGeometry(1.8, 0.02, 8, 64), new THREE.MeshBasicMaterial({color: 0xb967ff, transparent: true, opacity: 0.5}));
            ring.rotation.x = Math.PI / 3;
            scene.add(ring);
            if (window._bloomAnimId) cancelAnimationFrame(window._bloomAnimId);
            function animate() {
                window._bloomAnimId = requestAnimationFrame(animate);
                bust.rotation.y += 0.005; bust.rotation.x += 0.002;
                inner.rotation.y -= 0.008; inner.rotation.z += 0.003;
                ring.rotation.z += 0.003;
                renderer.render(scene, camera);
            }
            animate();
        }, 500);
    }
    </script>"#.to_string()
}

fn get_client_scripts(current_user_id: &str) -> String {
    let escaped_id = html_escape(current_user_id);
    format!(r#"<script>
    // WebSocket for real-time chat
    var ws;
    var currentUserId = '{escaped_id}';
    function connectWs() {{
        ws = new WebSocket('ws://' + location.host + '/ws?user_id=' + currentUserId);
        ws.onmessage = function(e) {{
            var msg = JSON.parse(e.data);
            if (msg.type === 'chat') {{
                var container = document.getElementById('chat-messages-' + msg.from_id);
                if (container) {{
                    var div = document.createElement('div');
                    div.className = 'msg-bubble msg-them';
                    div.textContent = msg.content;
                    container.appendChild(div);
                    container.scrollTop = container.scrollHeight;
                }}
            }} else if (msg.type === 'new_post') {{
                var feed = document.getElementById('feed');
                if (feed) {{
                    var temp = document.createElement('div');
                    temp.innerHTML = msg.html;
                    feed.insertBefore(temp.firstElementChild, feed.firstChild);
                    htmx.process(feed.firstElementChild);
                }}
            }}
        }};
        ws.onclose = function() {{ setTimeout(connectWs, 3000); }};
    }}
    connectWs();

    function sendChat(toId, content) {{
        if (!content || !content.trim()) return;
        if (ws && ws.readyState === 1) {{
            ws.send(JSON.stringify({{type: 'chat', from_id: currentUserId, to_id: toId, content: content.trim()}}));
            var container = document.getElementById('chat-messages-' + toId);
            if (container) {{
                var div = document.createElement('div');
                div.className = 'msg-bubble msg-you';
                div.textContent = content.trim();
                container.appendChild(div);
                container.scrollTop = container.scrollHeight;
            }}
        }}
    }}

    // Notification dropdown toggle
    document.addEventListener('click', function(e) {{
        var bell = document.getElementById('notif-bell');
        var dropdown = document.getElementById('notif-dropdown');
        if (!bell || !dropdown) return;
        if (bell.contains(e.target)) {{
            dropdown.style.display = dropdown.style.display === 'none' ? 'block' : 'none';
        }} else if (!dropdown.contains(e.target)) {{
            dropdown.style.display = 'none';
        }}
    }});

    // File upload preview
    document.addEventListener('change', function(e) {{
        if (e.target.type === 'file' && e.target.name === 'media') {{
            var preview = e.target.closest('form').querySelector('.upload-preview');
            if (preview && e.target.files[0]) {{
                var img = document.createElement('img');
                img.style.cssText = 'max-height:100px;border-radius:8px;margin-top:8px';
                var reader = new FileReader();
                reader.onload = function(ev) {{ img.src = ev.target.result; }};
                reader.readAsDataURL(e.target.files[0]);
                preview.textContent = '';
                preview.appendChild(img);
            }}
        }}
    }});

    // Process HTMX responses for dynamic content
    document.body.addEventListener('htmx:afterSwap', function(e) {{
        htmx.process(e.detail.target);
    }});
    </script>"#)
}

/// Base vaporwave CSS + interactive element styles
fn get_dynamic_style() -> String {
    // Include the base vaporwave CSS extracted from the hot-swap module
    let base_css = include_str!("hotswap/base_style.css");
    let interactive_css = r#"<style>
    .comment-form {
        display: flex;
        gap: 8px;
        padding: 8px 16px 12px;
        align-items: center;
    }
    .comment-input {
        flex: 1;
        background: rgba(255,255,255,0.04);
        border: 1px solid rgba(255,113,206,0.15);
        border-radius: 16px;
        padding: 6px 14px;
        color: #ffd1dc;
        font-family: 'VT323', monospace;
        font-size: 15px;
        outline: none;
    }
    .comment-input:focus { border-color: #01cdfe; box-shadow: 0 0 8px rgba(1,205,254,0.2); }
    .comment-submit {
        background: linear-gradient(135deg, #ff71ce, #b967ff);
        border: none; border-radius: 50%;
        width: 30px; height: 30px;
        color: white; cursor: pointer; font-size: 16px;
        display: flex; align-items: center; justify-content: center;
    }
    .comments-section { padding: 0 16px; max-height: 200px; overflow-y: auto; }
    .comment-item {
        display: flex; gap: 6px; align-items: baseline;
        padding: 4px 0; font-size: 14px;
        border-bottom: 1px solid rgba(255,255,255,0.03);
    }
    .comment-author-emoji { font-size: 14px; }
    .comment-author-name { color: #fff; font-weight: bold; font-size: 13px; }
    .comment-text { color: #ffd1dc; flex: 1; }
    .comment-time { color: rgba(255,209,220,0.4); font-size: 12px; }
    .post-form-card { margin-bottom: 16px; padding: 16px; }
    .post-textarea {
        width: 100%;
        background: rgba(255,255,255,0.04);
        border: 1px solid rgba(255,113,206,0.15);
        border-radius: 12px;
        padding: 12px;
        color: #ffd1dc;
        font-family: 'VT323', monospace;
        font-size: 17px;
        resize: vertical;
        outline: none;
        min-height: 60px;
    }
    .post-textarea:focus { border-color: #01cdfe; box-shadow: 0 0 12px rgba(1,205,254,0.2); }
    .post-textarea::placeholder { color: rgba(255,209,220,0.4); }
    .post-form-actions { display: flex; justify-content: space-between; align-items: center; margin-top: 10px; }
    .upload-label {
        cursor: pointer; padding: 6px 14px; border-radius: 16px;
        border: 1px solid rgba(255,113,206,0.2);
        font-size: 14px; color: #ffd1dc; transition: all 0.2s;
    }
    .upload-label:hover { background: rgba(255,113,206,0.1); border-color: #ff71ce; }
    .post-submit-btn {
        padding: 8px 20px; border-radius: 20px; border: none;
        background: linear-gradient(135deg, #ff71ce, #b967ff);
        color: white; font-family: 'VT323', monospace; font-size: 16px;
        cursor: pointer; transition: box-shadow 0.3s;
    }
    .post-submit-btn:hover { box-shadow: 0 0 15px rgba(255,113,206,0.5); }
    .reaction-active {
        background: rgba(255,113,206,0.2) !important;
        border-color: #ff71ce !important;
        box-shadow: 0 0 8px rgba(255,113,206,0.3);
        color: #fff !important;
    }
    .friend-status-btn {
        padding: 6px 14px; border-radius: 16px; font-size: 13px;
        color: #05ffa1; border: 1px solid rgba(5,255,161,0.3);
    }
    .friend-status-btn.pending { color: #ffd1dc; border-color: rgba(255,209,220,0.2); }
    .chat-active-header {
        padding: 8px 14px; border-bottom: 1px solid rgba(255,113,206,0.15);
        cursor: pointer; font-size: 14px; color: #01cdfe;
    }
    .chat-active-header:hover { background: rgba(1,205,254,0.05); }
    .messenger-panel.minimized .chat-messages,
    .messenger-panel.minimized .chat-list,
    .messenger-panel.minimized .chat-input-bar,
    .messenger-panel.minimized .chat-active-header { display: none; }
    .upload-preview { text-align: center; }
    .post-image img { display: block; }
    </style>"#;
    format!("{}\n{}", base_css, interactive_css)
}
