use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub id: i32,
    #[serde(default)]
    pub sender_name: String,
    pub text: String,
    #[serde(default)]
    #[serde(alias = "is_own")]
    pub is_outgoing: bool,
    #[serde(default)]
    pub timestamp: i64,
    #[serde(default)]
    pub is_file: bool,
    #[serde(default)]
    pub file_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Chat {
    pub id: i64,
    pub name: String,
    pub last_message: String,
    #[serde(default)]
    pub time: String,
    pub unread_count: i32,
    #[serde(default)]
    pub is_archived: bool,
    #[serde(default)]
    pub messages: Vec<Message>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Task {
    pub id: usize,
    pub user_name: String,
    pub text: String,
    pub created_at: String,
    pub completed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewMode {
    Chats,
    Tasks,
}

