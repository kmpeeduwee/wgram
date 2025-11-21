use grammers_client::{
    types::{Dialog, IterBuffer, Message},
    Client, SignInError,
};
use grammers_mtsender::SenderPool;
use grammers_session::storages::SqliteSession;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::task::JoinHandle;
use tracing::{info, warn};
use uuid::Uuid;

pub struct TelegramManager {
    api_id: i32,
    api_hash: String,
    session_path: String,
    session: Arc<SqliteSession>,
    client: Client,
    _runner_handle: JoinHandle<()>,
    sessions: HashMap<String, String>,
    pending_login_tokens: HashMap<String, grammers_client::types::LoginToken>,
    pending_password_tokens: HashMap<String, grammers_client::types::PasswordToken>,
    chat_map: HashMap<i64, grammers_client::types::Peer>,
}

impl TelegramManager {
    pub async fn new() -> Result<Self, anyhow::Error> {
        let api_id: i32 = std::env::var("TELEGRAM_API_ID")
            .unwrap_or_else(|_| {
                warn!("TELEGRAM_API_ID not set, using placeholder");
                "123456".to_string()
            })
            .parse()
            .unwrap_or(123456);

        let api_hash = std::env::var("TELEGRAM_API_HASH").unwrap_or_else(|_| {
            warn!("TELEGRAM_API_HASH not set, using placeholder");
            "placeholder_hash".to_string()
        });

        let session_path =
            std::env::var("TELEGRAM_SESSION_FILE").unwrap_or_else(|_| "wgram.session".to_string());

        info!("Initializing Telegram manager (grammers-client 0.8)");
        info!("API ID: {}", api_id);
        info!("Session file: {}", session_path);

        let session = Arc::new(SqliteSession::open(&session_path)?);

        let pool = SenderPool::new(Arc::clone(&session), api_id);
        let client = Client::new(&pool);

        let SenderPool { runner, .. } = pool;
        let runner_handle = tokio::spawn(runner.run());

        info!("Telegram client initialized successfully");

        Ok(Self {
            api_id,
            api_hash,
            session_path,
            session,
            client,
            _runner_handle: runner_handle,
            sessions: HashMap::new(),
            pending_login_tokens: HashMap::new(),
            pending_password_tokens: HashMap::new(),
            chat_map: HashMap::new(),
        })
    }

    pub async fn send_code(&mut self, phone: &str) -> Result<(), anyhow::Error> {
        info!("Requesting login code for phone: {}", phone);

        let _ = self.client.is_authorized().await;

        let token = match self.client.request_login_code(phone, &self.api_hash).await {
            Ok(token) => token,
            Err(e) => {
                let error_msg = e.to_string();
                if error_msg.contains("AUTH_RESTART") {
                    warn!("AUTH_RESTART occurred, retrying login code request...");
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                    self.client.request_login_code(phone, &self.api_hash).await?
                } else {
                    return Err(anyhow::anyhow!("request error: {}", error_msg));
                }
            }
        };

        self.pending_login_tokens.insert(phone.to_string(), token);

        info!("✅ Login code requested successfully for phone: {}", phone);
        Ok(())
    }

    pub async fn verify_code(&mut self, phone: &str, code: &str) -> Result<String, anyhow::Error> {
        info!(
            "Verifying login code for phone: {} with code: {}",
            phone, code
        );

        let token = self.pending_login_tokens.remove(phone).ok_or_else(|| {
            anyhow::anyhow!("No pending auth for this phone. Request code first!")
        })?;

        match self.client.sign_in(&token, code).await {
            Ok(_user) => {
                let session_id = Uuid::new_v4().to_string();
                self.sessions.insert(session_id.clone(), phone.to_string());

                info!("✅ Authentication successful! Session ID: {}", session_id);
                Ok(session_id)
            }
            Err(SignInError::PasswordRequired(password_token)) => {
                info!("2FA password required for phone: {}", phone);

                self.pending_password_tokens
                    .insert(phone.to_string(), password_token);

                Err(anyhow::anyhow!(
                    "2FA password required. Please call verify_password."
                ))
            }
            Err(SignInError::SignUpRequired { .. }) => Err(anyhow::anyhow!(
                "This phone number is not registered. Please sign up first."
            )),
            Err(e) => Err(anyhow::anyhow!("Sign in failed: {}", e)),
        }
    }

    pub async fn verify_password(
        &mut self,
        phone: &str,
        password: &str,
    ) -> Result<String, anyhow::Error> {
        info!("Verifying 2FA password for phone: {}", phone);

        let token = self.pending_password_tokens.remove(phone).ok_or_else(|| {
            anyhow::anyhow!("No pending 2FA auth for this phone. Verify code first!")
        })?;

        match self.client.check_password(token, password).await {
            Ok(_user) => {
                let session_id = Uuid::new_v4().to_string();
                self.sessions.insert(session_id.clone(), phone.to_string());

                info!(
                    "✅ 2FA authentication successful! Session ID: {}",
                    session_id
                );
                Ok(session_id)
            }
            Err(e) => Err(anyhow::anyhow!("2FA password verification failed: {}", e)),
        }
    }

    pub async fn get_dialogs(&mut self) -> Result<Vec<Dialog>, anyhow::Error> {
        info!("Fetching dialogs...");
        let mut iter: IterBuffer<_, Dialog> = self.client.iter_dialogs();
        let mut dialogs = Vec::new();
        let mut index = 1i64;

        while let Some(dialog) = iter.next().await? {
            self.chat_map.insert(index, dialog.peer().clone());
            dialogs.push(dialog);
            index += 1;
        }

        info!("✅ Fetched {} dialogs", dialogs.len());
        Ok(dialogs)
    }

    pub async fn get_messages(
        &self,
        chat_id: i64,
        limit: usize,
    ) -> Result<Vec<Message>, anyhow::Error> {
        info!("Fetching messages for chat_id: {}", chat_id);

        let chat = self
            .chat_map
            .get(&chat_id)
            .ok_or_else(|| anyhow::anyhow!("Chat not found for id: {}", chat_id))?;

        let mut iter = self.client.iter_messages(chat);
        let mut messages = Vec::new();

        while let Some(msg) = iter.next().await? {
            messages.push(msg);
            if messages.len() >= limit {
                break;
            }
        }

        messages.reverse();

        info!(
            "✅ Fetched {} messages for chat_id: {}",
            messages.len(),
            chat_id
        );
        Ok(messages)
    }

    pub fn get_session(&self, session_id: &str) -> Option<&String> {
        self.sessions.get(session_id)
    }

    pub async fn send_message(&self, chat_id: i64, text: &str) -> Result<(), anyhow::Error> {
        info!("Sending message to chat_id: {}", chat_id);

        let chat = self
            .chat_map
            .get(&chat_id)
            .ok_or_else(|| anyhow::anyhow!("Chat not found for id: {}", chat_id))?;

        self.client.send_message(chat, text).await?;

        info!("✅ Message sent successfully to chat_id: {}", chat_id);
        Ok(())
    }

    pub async fn is_authorized(&self) -> Result<bool, anyhow::Error> {
        Ok(self.client.is_authorized().await?)
    }

    pub fn disconnect(&self) {
        info!("Disconnecting Telegram client");
        self.client.disconnect();
    }
}

impl Drop for TelegramManager {
    fn drop(&mut self) {
        info!("TelegramManager dropped, disconnecting...");
        self.client.disconnect();
    }
}
