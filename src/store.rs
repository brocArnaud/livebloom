use crate::models::*;
use chrono::Utc;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

/// Thread-safe shared social network store
#[derive(Clone)]
pub struct SocialStore {
    inner: Arc<RwLock<StoreInner>>,
    /// Broadcast channel for real-time events (WebSocket)
    pub broadcast_tx: broadcast::Sender<String>,
}

struct StoreInner {
    pub users: HashMap<UserId, User>,
    pub posts: Vec<Post>,
    pub messages: Vec<Message>,
    pub notifications: Vec<Notification>,
    /// Tracks which user_id index maps to which user for quick lookup
    pub user_order: Vec<UserId>,
}

impl SocialStore {
    pub fn new_with_seed() -> Self {
        let (broadcast_tx, _) = broadcast::channel(256);
        let mut inner = StoreInner {
            users: HashMap::new(),
            posts: Vec::new(),
            messages: Vec::new(),
            notifications: Vec::new(),
            user_order: Vec::new(),
        };

        // Seed users
        let users_data = [
            ("AestheticDreamer", "🌸", "Living in the aesthetic"),
            ("NeonSunset_99", "🌅", "Chasing neon dreams"),
            ("VaporMike", "🎵", "Macintosh Plus on repeat"),
            ("CyberSakura", "🌸", "Digital cherry blossoms"),
            ("RetroWave_Luna", "🌙", "Moonlit synthwave"),
            ("PixelPhantom", "👾", "8-bit soul in a 4K world"),
        ];

        let mut user_ids = Vec::new();
        for (username, emoji, status) in &users_data {
            let id = Uuid::new_v4();
            user_ids.push(id);
            inner.user_order.push(id);
            inner.users.insert(
                id,
                User {
                    id,
                    username: username.to_string(),
                    emoji: emoji.to_string(),
                    status: status.to_string(),
                    friends: HashSet::new(),
                    friend_requests_in: HashSet::new(),
                    friend_requests_out: HashSet::new(),
                },
            );
        }

        // Make some friendships (user 0 is friends with 1, 2, 3)
        for &friend_idx in &[1, 2, 3] {
            let u0 = user_ids[0];
            let uf = user_ids[friend_idx];
            inner.users.get_mut(&u0).unwrap().friends.insert(uf);
            inner.users.get_mut(&uf).unwrap().friends.insert(u0);
        }

        // Seed posts (newest first)
        let posts_data: Vec<(usize, &str, Option<&str>, Option<&str>)> = vec![
            (1, "Just watched the sun dissolve into pixels 🌅", None,
             Some("linear-gradient(180deg, #ff6b6b 0%, #ffa07a 40%, #ff71ce 70%, #b967ff 100%)")),
            (2, "リサフランク420 / 現代のコンピュー 🎵\n\nThis track changed everything. Macintosh Plus forever.", None, None),
            (3, "Found this abandoned mall. The aesthetics are unreal...", None,
             Some("linear-gradient(180deg, #0f0c29 0%, #302b63 50%, #24243e 100%)")),
            (0, "Late night coding in the v a p o r w a v e ✨\n\nWhen the terminal glows pink, you know it's going to be a good session.", None, None),
            (4, "The moon looks different in 16-bit 🌙", None,
             Some("linear-gradient(180deg, #0c0032 0%, #190061 30%, #240090 50%, #3500d3 70%, #282828 100%)")),
            (5, "New high score on a game that doesn't exist 👾\n\nScore: 999,999,999\nLevel: ∞\nLives: Yes", None, None),
            (2, "My room at 3am. Pure aesthetic.", None,
             Some("linear-gradient(180deg, #12002e 0%, #1a0040 30%, #2d1060 50%, #ff71ce 90%, #ff71ce 100%)")),
            (1, "Remember when the future was now?\n\nIt still is. We're living in it. Every pixel, every glow, every beat. This is the future our past dreamed of. 🌐", None, None),
        ];

        let reaction_names = ["like", "love", "laugh", "wow", "bloom"];

        for (i, (author_idx, content, media, gradient)) in posts_data.iter().enumerate() {
            let post_id = Uuid::new_v4();
            let mut reactions: HashMap<String, HashSet<UserId>> = HashMap::new();

            // Deterministic reactions from other users
            for (ri, rname) in reaction_names.iter().enumerate() {
                let mut set = HashSet::new();
                let count = ((i * 17 + ri * 31 + 7) % 89) + 1;
                for u in 0..std::cmp::min(count, user_ids.len()) {
                    set.insert(user_ids[u % user_ids.len()]);
                }
                // Vary which users reacted
                for extra in 0..((count / 6) % 4) {
                    set.insert(user_ids[(extra + ri + i) % user_ids.len()]);
                }
                reactions.insert(rname.to_string(), set);
            }

            let hours_ago = (i as i64 + 1) * 3;
            let created = Utc::now() - chrono::Duration::hours(hours_ago);

            inner.posts.push(Post {
                id: post_id,
                author_id: user_ids[*author_idx],
                content: content.to_string(),
                media_url: media.map(|s| s.to_string()),
                css_gradient: gradient.map(|s| s.to_string()),
                reactions,
                comments: Vec::new(),
                created_at: created,
            });
        }

        // Seed some comments
        let first_post_id = inner.posts[0].id;
        inner.posts[0].comments.push(Comment {
            id: Uuid::new_v4(),
            author_id: user_ids[2],
            content: "Those colors are unreal! 🎨".to_string(),
            created_at: Utc::now() - chrono::Duration::hours(1),
        });
        inner.posts[0].comments.push(Comment {
            id: Uuid::new_v4(),
            author_id: user_ids[3],
            content: "A E S T H E T I C".to_string(),
            created_at: Utc::now() - chrono::Duration::minutes(30),
        });

        // Seed some messages
        let chat_messages = [
            (1, 0, "hey, did you see the new vapor collection? 🌸"),
            (0, 1, "omg yes! the sunset series is incredible"),
            (1, 0, "right?? the colors are so surreal"),
            (0, 1, "we should go to that abandoned mall this weekend 📸"),
            (1, 0, "I'm in! let's chase that aesthetic ✨"),
        ];
        for (fi, ti, content) in &chat_messages {
            inner.messages.push(Message {
                id: Uuid::new_v4(),
                from_id: user_ids[*fi],
                to_id: user_ids[*ti],
                content: content.to_string(),
                created_at: Utc::now() - chrono::Duration::minutes(30 - inner.messages.len() as i64 * 5),
            });
        }

        // Seed notifications for user 0
        let notifs = [
            (1, NotifKind::Liked(first_post_id), 2),
            (2, NotifKind::FriendRequest, 60),
            (3, NotifKind::Commented(first_post_id), 180),
            (4, NotifKind::Shared(first_post_id), 1440),
            (5, NotifKind::Followed, 2880),
        ];
        for (from_idx, kind, mins_ago) in notifs {
            inner.notifications.push(Notification {
                id: Uuid::new_v4(),
                user_id: user_ids[0],
                from_user_id: user_ids[from_idx],
                kind,
                read: mins_ago > 200,
                created_at: Utc::now() - chrono::Duration::minutes(mins_ago),
            });
        }

        Self {
            inner: Arc::new(RwLock::new(inner)),
            broadcast_tx,
        }
    }

    // ── User operations ──

    pub async fn get_user(&self, id: &UserId) -> Option<User> {
        let inner = self.inner.read().await;
        inner.users.get(id).cloned()
    }

    pub async fn get_all_users(&self) -> Vec<User> {
        let inner = self.inner.read().await;
        inner.user_order.iter().filter_map(|id| inner.users.get(id).cloned()).collect()
    }

    pub async fn current_user_id(&self) -> UserId {
        let inner = self.inner.read().await;
        inner.user_order[0]
    }

    pub async fn get_non_friends(&self, user_id: &UserId) -> Vec<User> {
        let inner = self.inner.read().await;
        let user = match inner.users.get(user_id) {
            Some(u) => u,
            None => return Vec::new(),
        };
        inner
            .users
            .values()
            .filter(|u| u.id != *user_id && !user.friends.contains(&u.id))
            .cloned()
            .collect()
    }

    // ── Post operations ──

    pub async fn get_posts(&self) -> Vec<Post> {
        let inner = self.inner.read().await;
        inner.posts.clone()
    }

    pub async fn create_post(
        &self,
        author_id: UserId,
        content: String,
        media_url: Option<String>,
    ) -> Post {
        let mut inner = self.inner.write().await;
        let post = Post {
            id: Uuid::new_v4(),
            author_id,
            content,
            media_url,
            css_gradient: None,
            reactions: HashMap::new(),
            comments: Vec::new(),
            created_at: Utc::now(),
        };
        inner.posts.insert(0, post.clone());
        post
    }

    pub async fn toggle_reaction(
        &self,
        post_id: &PostId,
        user_id: &UserId,
        reaction_type: &str,
    ) -> Option<Post> {
        let mut inner = self.inner.write().await;
        let post_idx = inner.posts.iter().position(|p| p.id == *post_id)?;

        let set = inner.posts[post_idx]
            .reactions
            .entry(reaction_type.to_string())
            .or_default();
        let added = if set.contains(user_id) {
            set.remove(user_id);
            false
        } else {
            set.insert(*user_id);
            true
        };

        let post_author = inner.posts[post_idx].author_id;
        let pid = inner.posts[post_idx].id;

        if added && post_author != *user_id {
            inner.notifications.insert(
                0,
                Notification {
                    id: Uuid::new_v4(),
                    user_id: post_author,
                    from_user_id: *user_id,
                    kind: NotifKind::Liked(pid),
                    read: false,
                    created_at: Utc::now(),
                },
            );
        }
        Some(inner.posts[post_idx].clone())
    }

    pub async fn add_comment(
        &self,
        post_id: &PostId,
        author_id: UserId,
        content: String,
    ) -> Option<(Comment, Post)> {
        let mut inner = self.inner.write().await;
        let post_idx = inner.posts.iter().position(|p| p.id == *post_id)?;
        let comment = Comment {
            id: Uuid::new_v4(),
            author_id,
            content,
            created_at: Utc::now(),
        };
        inner.posts[post_idx].comments.push(comment.clone());

        let post_author = inner.posts[post_idx].author_id;
        let pid = inner.posts[post_idx].id;

        if post_author != author_id {
            inner.notifications.insert(
                0,
                Notification {
                    id: Uuid::new_v4(),
                    user_id: post_author,
                    from_user_id: author_id,
                    kind: NotifKind::Commented(pid),
                    read: false,
                    created_at: Utc::now(),
                },
            );
        }
        Some((comment, inner.posts[post_idx].clone()))
    }

    // ── Friend operations ──

    pub async fn send_friend_request(&self, from_id: &UserId, to_id: &UserId) -> bool {
        let mut inner = self.inner.write().await;
        match inner.users.get_mut(from_id) {
            Some(u) => {
                u.friend_requests_out.insert(*to_id);
            }
            None => return false,
        };
        if let Some(to) = inner.users.get_mut(to_id) {
            to.friend_requests_in.insert(*from_id);
        }
        inner.notifications.insert(
            0,
            Notification {
                id: Uuid::new_v4(),
                user_id: *to_id,
                from_user_id: *from_id,
                kind: NotifKind::FriendRequest,
                read: false,
                created_at: Utc::now(),
            },
        );
        true
    }

    pub async fn accept_friend_request(&self, user_id: &UserId, friend_id: &UserId) -> bool {
        let mut inner = self.inner.write().await;
        match inner.users.get_mut(user_id) {
            Some(u) => {
                u.friend_requests_in.remove(friend_id);
                u.friends.insert(*friend_id);
            }
            None => return false,
        };
        if let Some(friend) = inner.users.get_mut(friend_id) {
            friend.friend_requests_out.remove(user_id);
            friend.friends.insert(*user_id);
        }
        inner.notifications.insert(
            0,
            Notification {
                id: Uuid::new_v4(),
                user_id: *friend_id,
                from_user_id: *user_id,
                kind: NotifKind::FriendAccepted,
                read: false,
                created_at: Utc::now(),
            },
        );
        true
    }

    pub async fn get_friend_status(&self, user_id: &UserId, other_id: &UserId) -> FriendStatus {
        let inner = self.inner.read().await;
        let user = match inner.users.get(user_id) {
            Some(u) => u,
            None => return FriendStatus::None,
        };
        if user.friends.contains(other_id) {
            FriendStatus::Friends
        } else if user.friend_requests_out.contains(other_id) {
            FriendStatus::RequestSent
        } else if user.friend_requests_in.contains(other_id) {
            FriendStatus::RequestReceived
        } else {
            FriendStatus::None
        }
    }

    // ── Message operations ──

    pub async fn get_conversation(&self, user_a: &UserId, user_b: &UserId) -> Vec<Message> {
        let inner = self.inner.read().await;
        inner
            .messages
            .iter()
            .filter(|m| {
                (m.from_id == *user_a && m.to_id == *user_b)
                    || (m.from_id == *user_b && m.to_id == *user_a)
            })
            .cloned()
            .collect()
    }

    pub async fn send_message(
        &self,
        from_id: UserId,
        to_id: UserId,
        content: String,
    ) -> Message {
        let mut inner = self.inner.write().await;
        let msg = Message {
            id: Uuid::new_v4(),
            from_id,
            to_id,
            content,
            created_at: Utc::now(),
        };
        inner.messages.push(msg.clone());
        msg
    }

    /// Get list of users the current user has conversations with
    pub async fn get_chat_list(&self, user_id: &UserId) -> Vec<(User, Option<Message>)> {
        let inner = self.inner.read().await;
        let mut chat_partners: HashMap<UserId, Message> = HashMap::new();
        for msg in &inner.messages {
            let partner = if msg.from_id == *user_id {
                msg.to_id
            } else if msg.to_id == *user_id {
                msg.from_id
            } else {
                continue;
            };
            let existing = chat_partners.get(&partner);
            if existing.is_none() || existing.unwrap().created_at < msg.created_at {
                chat_partners.insert(partner, msg.clone());
            }
        }
        let mut result: Vec<(User, Option<Message>)> = chat_partners
            .into_iter()
            .filter_map(|(uid, msg)| {
                inner.users.get(&uid).map(|u| (u.clone(), Some(msg)))
            })
            .collect();
        result.sort_by(|a, b| {
            b.1.as_ref()
                .map(|m| m.created_at)
                .cmp(&a.1.as_ref().map(|m| m.created_at))
        });
        result
    }

    // ── Notification operations ──

    pub async fn get_notifications(&self, user_id: &UserId) -> Vec<(Notification, User)> {
        let inner = self.inner.read().await;
        inner
            .notifications
            .iter()
            .filter(|n| n.user_id == *user_id)
            .filter_map(|n| {
                inner
                    .users
                    .get(&n.from_user_id)
                    .map(|u| (n.clone(), u.clone()))
            })
            .take(10)
            .collect()
    }

    pub async fn unread_count(&self, user_id: &UserId) -> usize {
        let inner = self.inner.read().await;
        inner
            .notifications
            .iter()
            .filter(|n| n.user_id == *user_id && !n.read)
            .count()
    }

    pub async fn mark_notifications_read(&self, user_id: &UserId) {
        let mut inner = self.inner.write().await;
        for n in inner.notifications.iter_mut() {
            if n.user_id == *user_id {
                n.read = true;
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FriendStatus {
    Friends,
    RequestSent,
    RequestReceived,
    None,
}
