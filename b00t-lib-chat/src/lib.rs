//! # b00t chat
//!
//! A lightweight coordination channel for PromptExecution agents. The chat
//! pipeline exposes a local Unix domain socket that agents can use to exchange
//! JSON messages while optionally mirroring the payloads to NATS (currently
//! implemented as telemetry stubs).
//!
//! ## Highlights
//!
//! - Local IPC **named pipe** at `~/.b00t/chat.channel.socket`
//! - Simple JSON [`ChatMessage`](crate::message::ChatMessage) envelope
//! - Async client for CLI usage via [`ChatClient`](crate::client::ChatClient)
//! - Inbox utilities that let MCP servers surface unread notifications
//!
//! ```no_run
//! use b00t_chat::{ChatClient, ChatMessage};
//!
//! # #[tokio::main]
//! # async fn main() -> anyhow::Result<()> {
//! let client = ChatClient::local_default()?;
//! let message = ChatMessage::new("mission.alpha", "frontend", "UI ready for review");
//! client.send(&message).await?;
//! # Ok(())
//! # }
//! ```

pub mod client;
pub mod error;
pub mod message;
pub mod server;
pub mod transport;

pub use client::ChatClient;
pub use error::{ChatError, ChatResult};
pub use message::ChatMessage;
pub use server::{spawn_local_server, ChatInbox, LocalChatServer};
pub use transport::{default_socket_path, ChatTransport, ChatTransportConfig, ChatTransportKind};
