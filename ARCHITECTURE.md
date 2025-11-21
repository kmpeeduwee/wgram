# Wgram Project Architecture

## 📁 Structure

The project is organized following **Feature-Sliced Design** principles:

```
src/
├── shared/              # Reusable types and utilities
│   ├── types.rs         # Message, Chat, Task, ViewMode
│   ├── api/
│   │   └── websocket.rs # WsRequest, WsResponse
│   └── utils/
│       └── time.rs      # get_current_time, format_timestamp
│
├── widgets/             # UI components
│   ├── sidebar.rs       # Side navigation
│   ├── chat_list.rs     # Chat list with search
│   ├── chat_window.rs   # Chat window with messages
│   └── task_list.rs     # Task list
│
├── features/            # Business logic
│   ├── websocket.rs     # WebSocket (WS_REF is located here!)
│   ├── messaging.rs     # Send/receive messages
│   └── tasks.rs         # Task creation
│
├── app.rs               # Main component (~175 lines)
├── auth.rs              # Authentication
├── lib.rs               # Module exports
└── main.rs              # Entry point
```

## 🔧 Important Notes

### WebSocket Reference (WS_REF)

**IMPORTANT:** `WS_REF` (global WebSocket reference) is located **ONLY** in `features/websocket.rs`.

All modules that work with WebSocket must import it:
```rust
use crate::features::websocket::WS_REF;
```

**DO NOT create** duplicate `thread_local! { static WS_REF: ... }` in other modules!

### Hooks

**features/websocket.rs:**
```rust
pub fn use_websocket(...) -> ()
```
Creates and manages WebSocket connection, auto-updates messages.

**features/messaging.rs:**
```rust
pub fn use_messaging(...) -> (send_message, get_messages)
```
Returns functions for sending and receiving messages.

**features/tasks.rs:**
```rust
pub fn use_tasks(...) -> create_task
```
Returns function for creating tasks.

## Running

```bash
# Development (WASM + hot reload)
trunk serve

# Production build
trunk build --release

# Desktop (if available)
cargo run
```

## Checking

```bash
# Type checking
cargo check

# Build
cargo build

# Linter
cargo clippy
```

