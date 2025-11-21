# Wgram Backend

Backend server for Wgram - Telegram client with Tasks integration.

## Setup

### 1. Get Telegram API Credentials

1. Go to https://my.telegram.org/apps
2. Log in with your phone number
3. Click "API development tools"
4. Fill in the form:
   - App title: `Wgram`
   - Short name: `wgram`
   - Platform: `Desktop`
5. You'll get:
   - **api_id** (number)
   - **api_hash** (string)

### 2. Configure Environment Variables

Create a `.env` file in the project root (not in backend directory):

```bash
# Copy the example file
cp .env.example .env
```

Edit `.env` and set your credentials:

```bash
TELEGRAM_API_ID=123456
TELEGRAM_API_HASH=your_api_hash_here
TELEGRAM_SESSION_FILE=wgram.session
```

### 3. Run the Backend

```bash
cd backend
cargo run
```

The server will:
1. Automatically load `.env` from the project root
2. Start on `http://127.0.0.1:3000`

**Note**: The `.env` file must be in the project root directory (`/Users/kmpeeduwee/develop/wgram/.env`), not in the backend directory.

## API Endpoints

### Health Check
```
GET /health
```

### Request Authentication Code
```
POST /auth/request-code
Content-Type: application/json

{
  "phone": "+1234567890"
}
```

Response:
```json
{
  "success": true,
  "message": "Code sent! Check your SMS or email"
}
```

### Verify Code
```
POST /auth/verify-code
Content-Type: application/json

{
  "phone": "+1234567890",
  "code": "12345"
}
```

Response:
```json
{
  "success": true,
  "message": "Authenticated successfully!",
  "session_id": "uuid-here"
}
```

### WebSocket Connection
```
WS /ws
```

## How It Works

1. User enters phone number
2. Backend requests code from Telegram using `grammers`
3. Telegram sends code (SMS or email, depending on user settings)
4. User enters code
5. Backend verifies code with Telegram
6. Session created, user authenticated
7. WebSocket connection for real-time messages

## Notes

- Codes can arrive via SMS or email (if configured in Telegram settings)
- Session is stored in memory (will be lost on restart)
- TODO: Add database for persistent sessions
- TODO: Implement 2FA password support
