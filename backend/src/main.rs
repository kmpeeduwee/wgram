use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info};

mod telegram;
use telegram::TelegramManager;

#[derive(Clone)]
struct AppState {
    telegram: Arc<RwLock<TelegramManager>>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AuthRequest {
    phone: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct VerifyRequest {
    phone: String,
    code: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AuthResponse {
    success: bool,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    session_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum WsRequest {
    GetDialogs,
    GetMessages {
        chat_id: i64,
    },
    SendMessage {
        chat_id: i64,
        text: String,
    },
    SendFile {
        chat_id: i64,
        file_name: String,
        file_data: Vec<u8>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum WsResponse {
    Dialogs {
        data: Vec<FrontendDialog>,
    },
    Messages {
        chat_id: i64,
        data: Vec<FrontendMessage>,
    },
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
    NewMessage {
        chat_id: i64,
        message: FrontendMessage,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct FrontendDialog {
    id: i64,
    name: String,
    last_message: String,
    unread_count: i32,
    is_archived: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct FrontendMessage {
    id: i32,
    text: String,
    sender_name: String,
    is_outgoing: bool,
    timestamp: i64,
    #[serde(default)]
    is_file: bool,
    #[serde(default)]
    file_name: Option<String>,
}

async fn request_code(
    State(state): State<AppState>,
    Json(payload): Json<AuthRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    info!("Requesting code for phone: {}", payload.phone);

    let mut telegram = state.telegram.write().await;
    telegram.send_code(&payload.phone).await?;

    Ok(Json(AuthResponse {
        success: true,
        message: "Code sent! Check your SMS or email".to_string(),
        session_id: None,
    }))
}

async fn verify_code(
    State(state): State<AppState>,
    Json(payload): Json<VerifyRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    info!("Verifying code for phone: {}", payload.phone);

    let mut telegram = state.telegram.write().await;
    let session_id = telegram.verify_code(&payload.phone, &payload.code).await?;

    Ok(Json(AuthResponse {
        success: true,
        message: "Authenticated successfully!".to_string(),
        session_id: Some(session_id),
    }))
}

async fn handle_websocket(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(|socket| websocket_handler(socket, state))
}

async fn websocket_handler(mut socket: WebSocket, state: AppState) {
    info!("WebSocket connection established");

    while let Some(msg) = socket.recv().await {
        match msg {
            Ok(Message::Text(text)) => {
                info!("Received command: {}", text);
                let request: WsRequest = match serde_json::from_str(&text) {
                    Ok(req) => req,
                    Err(e) => {
                        error!("Failed to parse command: {}", e);
                        continue;
                    }
                };

                let response = match request {
                    WsRequest::GetDialogs => {
                        let mut telegram = state.telegram.write().await;

                        let dialogs_data = match telegram.is_authorized().await {
                            Ok(false) => {
                                error!("Telegram client is not authorized!");
                                vec![]
                            }
                            Err(e) => {
                                error!("Failed to check authorization: {}", e);
                                vec![]
                            }
                            Ok(true) => {
                                info!("Telegram client is authorized, fetching dialogs...");
                                match telegram.get_dialogs().await {
                                    Ok(dialogs) => {
                                        info!("Successfully fetched {} dialogs", dialogs.len());
                                        let frontend_dialogs: Vec<FrontendDialog> = dialogs
                                            .into_iter()
                                            .enumerate()
                                            .map(|(index, d)| {
                                                let id = (index + 1) as i64;
                                                let name = d.peer.name().map(|s| s.to_string()).unwrap_or_else(|| "Unknown".to_string());

                                                let (unread_count, is_archived) = if let grammers_client::grammers_tl_types::enums::Dialog::Dialog(dialog) = &d.raw {
                                                    let archived = dialog.folder_id.unwrap_or(0) == 1;
                                                    (dialog.unread_count, archived)
                                                } else {
                                                    (0, false)
                                                };

                                                let last_message = d
                                                    .last_message
                                                    .as_ref()
                                                    .map(|m| m.text().to_string())
                                                    .unwrap_or_default();

                                                info!("Dialog: id={}, name={}, last_message={}, unread={}, archived={}",
                                                    id, name, last_message, unread_count, is_archived);

                                                FrontendDialog {
                                                    id,
                                                    name,
                                                    last_message,
                                                    unread_count,
                                                    is_archived,
                                                }
                                            })
                                            .collect();

                                        info!(
                                            "Sending {} dialogs to frontend",
                                            frontend_dialogs.len()
                                        );
                                        frontend_dialogs
                                    }
                                    Err(e) => {
                                        error!("Failed to get dialogs: {}", e);
                                        vec![]
                                    }
                                }
                            }
                        };

                        WsResponse::Dialogs { data: dialogs_data }
                    }
                    WsRequest::GetMessages { chat_id } => {
                        let telegram = state.telegram.read().await;

                        match telegram.get_messages(chat_id, 50).await {
                            Ok(messages) => {
                                info!(
                                    "Successfully fetched {} messages for chat_id: {}",
                                    messages.len(),
                                    chat_id
                                );
                                let frontend_messages: Vec<FrontendMessage> = messages
                                    .into_iter()
                                    .map(|m| {
                                        let sender_name = m
                                            .sender()
                                            .and_then(|s| s.name())
                                            .map(|s| s.to_string())
                                            .unwrap_or_else(|| "Unknown".to_string());

                                        FrontendMessage {
                                            id: m.id(),
                                            text: m.text().to_string(),
                                            sender_name,
                                            is_outgoing: m.outgoing(),
                                            timestamp: m.date().timestamp(),
                                            is_file: false,
                                            file_name: None,
                                        }
                                    })
                                    .collect();

                                WsResponse::Messages {
                                    chat_id,
                                    data: frontend_messages,
                                }
                            }
                            Err(e) => {
                                error!("Failed to get messages: {}", e);
                                WsResponse::Messages {
                                    chat_id,
                                    data: vec![],
                                }
                            }
                        }
                    }
                    WsRequest::SendMessage { chat_id, text } => {
                        let telegram = state.telegram.read().await;

                        match telegram.send_message(chat_id, &text).await {
                            Ok(()) => {
                                info!("Message sent successfully to chat_id: {}", chat_id);
                                WsResponse::MessageSent {
                                    chat_id,
                                    success: true,
                                    message: "Message sent successfully".to_string(),
                                }
                            }
                            Err(e) => {
                                error!("Failed to send message: {}", e);
                                WsResponse::MessageSent {
                                    chat_id,
                                    success: false,
                                    message: format!("Failed to send message: {}", e),
                                }
                            }
                        }
                    }
                    WsRequest::SendFile {
                        chat_id,
                        file_name: _,
                        file_data: _,
                    } => {
                        error!("File sending not yet implemented");
                        WsResponse::FileSent {
                            chat_id,
                            success: false,
                            message: "File sending not yet implemented".to_string(),
                        }
                    }
                };

                let response_text = serde_json::to_string(&response).unwrap();
                if let Err(e) = socket.send(Message::Text(response_text)).await {
                    error!("Failed to send response: {}", e);
                    break;
                }
            }
            Ok(_) => {}
            Err(e) => {
                error!("WebSocket error: {}", e);
                break;
            }
        }
    }

    info!("WebSocket connection closed");
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    info!("Starting Wgram backend server...");

    match dotenvy::from_path("../.env") {
        Ok(_) => info!("Loaded environment variables from .env file"),
        Err(_) => info!("No .env file found, using system environment variables"),
    }

    let telegram = TelegramManager::new()
        .await
        .expect("Failed to initialize Telegram client");

    let app_state = AppState {
        telegram: Arc::new(RwLock::new(telegram)),
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/auth/request-code", post(request_code))
        .route("/auth/verify-code", post(verify_code))
        .route("/ws", get(handle_websocket))
        .layer(cors)
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    info!("Backend server listening on http://127.0.0.1:3000");

    axum::serve(listener, app).await.unwrap();
}

#[derive(Debug)]
struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        error!("Application error: {:?}", self.0);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(AuthResponse {
                success: false,
                message: self.0.to_string(),
                session_id: None,
            }),
        )
            .into_response()
    }
}

impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
