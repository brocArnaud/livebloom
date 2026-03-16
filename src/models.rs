use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

pub type UserId = Uuid;
pub type PostId = Uuid;
pub type CommentId = Uuid;
pub type MessageId = Uuid;
pub type NotifId = Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub username: String,
    pub emoji: String,
    pub status: String,
    pub friends: HashSet<UserId>,
    pub friend_requests_in: HashSet<UserId>,
    pub friend_requests_out: HashSet<UserId>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Post {
    pub id: PostId,
    pub author_id: UserId,
    pub content: String,
    pub media_url: Option<String>,
    pub css_gradient: Option<String>,
    pub reactions: HashMap<String, HashSet<UserId>>,
    pub comments: Vec<Comment>,
    pub created_at: DateTime<Utc>,
}

impl Post {
    pub fn reaction_count(&self, rtype: &str) -> usize {
        self.reactions.get(rtype).map_or(0, |s| s.len())
    }

    pub fn total_reactions(&self) -> usize {
        self.reactions.values().map(|s| s.len()).sum()
    }

    pub fn user_reacted(&self, user_id: &UserId, rtype: &str) -> bool {
        self.reactions
            .get(rtype)
            .is_some_and(|s| s.contains(user_id))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Comment {
    pub id: CommentId,
    pub author_id: UserId,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub id: MessageId,
    pub from_id: UserId,
    pub to_id: UserId,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Notification {
    pub id: NotifId,
    pub user_id: UserId,
    pub from_user_id: UserId,
    pub kind: NotifKind,
    pub read: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum NotifKind {
    Liked(PostId),
    Commented(PostId),
    FriendRequest,
    FriendAccepted,
    Shared(PostId),
    Followed,
}

impl NotifKind {
    pub fn action_text(&self) -> &str {
        match self {
            Self::Liked(_) => "liked your post",
            Self::Commented(_) => "commented on your post",
            Self::FriendRequest => "sent you a friend request",
            Self::FriendAccepted => "accepted your friend request",
            Self::Shared(_) => "shared your post",
            Self::Followed => "started following you",
        }
    }
}

pub const REACTION_TYPES: &[(&str, &str)] = &[
    ("like", "👍"),
    ("love", "❤️"),
    ("laugh", "😂"),
    ("wow", "😮"),
    ("bloom", "🌸"),
];

/// WebSocket message between server and client
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsMessage {
    #[serde(rename = "chat")]
    Chat {
        from_id: String,
        to_id: String,
        content: String,
    },
    #[serde(rename = "notification")]
    Notification { html: String },
    #[serde(rename = "new_post")]
    NewPost { html: String },
}
