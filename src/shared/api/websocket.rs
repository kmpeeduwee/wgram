use serde::{Deserialize, Serialize};
use crate::shared::types::{Chat, Message};

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum WsRequest {
    GetDialogs,
    GetMessages { chat_id: i64 },
    SendMessage { chat_id: i64, text: String },
    SendFile {
        chat_id: i64,
        file_name: String,
        file_data: Vec<u8>,
    },
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum WsResponse {
    Dialogs { data: Vec<Chat> },
    Messages { chat_id: i64, data: Vec<Message> },
    MessageSent {
        chat_id: i64,
        success: bool,
        message: String,
    },
    FileSent {
        chat_id: i64,
        success: bool,
        message: String,
    },
    NewMessage { chat_id: i64, message: Message },
}

