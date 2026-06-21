use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde::{Deserialize, Serialize};

/// Role of a message in a conversation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

/// A single message in a conversation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConversationMessage {
    pub role: MessageRole,
    pub content: String,
    pub timestamp_ms: u64,
    pub model: Option<String>,
    pub token_count: Option<u32>,
}

/// Conversation session with context window management.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConversationSession {
    pub id: String,
    pub product: String,
    pub created_at_ms: u64,
    pub updated_at_ms: u64,
    pub messages: Vec<ConversationMessage>,
    pub metadata: HashMap<String, String>,
    pub max_context_tokens: u32,
    pub system_prompt: Option<String>,
}

impl ConversationSession {
    pub fn new(id: String, product: String, max_context_tokens: u32) -> Self {
        let now = now_ms();
        Self {
            id,
            product,
            created_at_ms: now,
            updated_at_ms: now,
            messages: Vec::new(),
            metadata: HashMap::new(),
            max_context_tokens,
            system_prompt: None,
        }
    }

    /// Add a message and trim context window if needed.
    pub fn add_message(&mut self, role: MessageRole, content: String, token_count: Option<u32>) {
        let now = now_ms();
        self.updated_at_ms = now;
        self.messages.push(ConversationMessage {
            role,
            content,
            timestamp_ms: now,
            model: None,
            token_count,
        });
        self.trim_context();
    }

    /// Get messages formatted for an API request, respecting context window.
    pub fn get_context_messages(&self) -> Vec<&ConversationMessage> {
        let mut total_tokens: u32 = 0;
        let mut result = Vec::new();

        // Walk backwards from newest messages
        for msg in self.messages.iter().rev() {
            let tokens = msg.token_count.unwrap_or(estimate_tokens(&msg.content));
            if total_tokens + tokens > self.max_context_tokens && !result.is_empty() {
                break;
            }
            total_tokens += tokens;
            result.push(msg);
        }

        result.reverse();
        result
    }

    /// Trim oldest messages to fit within context window.
    fn trim_context(&mut self) {
        let mut total: u32 = 0;
        for msg in &self.messages {
            total += msg.token_count.unwrap_or(estimate_tokens(&msg.content));
        }

        while total > self.max_context_tokens && self.messages.len() > 1 {
            let removed = self.messages.remove(0);
            total -= removed
                .token_count
                .unwrap_or(estimate_tokens(&removed.content));
        }
    }

    /// Get total token count for all messages.
    pub fn total_tokens(&self) -> u32 {
        self.messages
            .iter()
            .map(|m| m.token_count.unwrap_or(estimate_tokens(&m.content)))
            .sum()
    }

    /// Clear all messages but keep session metadata.
    pub fn clear(&mut self) {
        self.messages.clear();
        self.updated_at_ms = now_ms();
    }
}

/// Rough token estimation: ~4 characters per token.
fn estimate_tokens(text: &str) -> u32 {
    (text.len() as u32 / 4).max(1)
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Manages multiple conversation sessions with expiration.
#[derive(Clone, Debug, Default)]
pub struct SessionManager {
    sessions: Arc<Mutex<HashMap<String, ConversationSession>>>,
    default_max_tokens: u32,
    session_ttl: Duration,
}

impl SessionManager {
    pub fn new(default_max_tokens: u32, session_ttl: Duration) -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            default_max_tokens,
            session_ttl,
        }
    }

    pub fn with_defaults() -> Self {
        Self::new(8192, Duration::from_secs(3600))
    }

    /// Create a new session.
    pub fn create_session(&self, id: String, product: String) -> ConversationSession {
        let session = ConversationSession::new(id, product, self.default_max_tokens);
        self.sessions
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(session.id.clone(), session.clone());
        session
    }

    /// Create a session with custom context window.
    pub fn create_session_with_tokens(
        &self,
        id: String,
        product: String,
        max_tokens: u32,
    ) -> ConversationSession {
        let session = ConversationSession::new(id, product, max_tokens);
        self.sessions
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(session.id.clone(), session.clone());
        session
    }

    /// Get a session by ID.
    pub fn get_session(&self, id: &str) -> Option<ConversationSession> {
        self.sessions
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(id)
            .cloned()
    }

    /// Update a session.
    pub fn update_session(&self, session: ConversationSession) {
        self.sessions
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(session.id.clone(), session);
    }

    /// Delete a session.
    pub fn delete_session(&self, id: &str) -> bool {
        self.sessions
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .remove(id)
            .is_some()
    }

    /// List all active sessions.
    pub fn list_sessions(&self) -> Vec<ConversationSession> {
        self.sessions
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .values()
            .cloned()
            .collect()
    }

    /// Remove expired sessions.
    pub fn cleanup_expired(&self) -> usize {
        let now = now_ms();
        let ttl_ms = self.session_ttl.as_millis() as u64;
        let mut sessions = self
            .sessions
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let before = sessions.len();
        sessions.retain(|_, s| now - s.updated_at_ms < ttl_ms);
        before - sessions.len()
    }

    /// Get session count.
    pub fn session_count(&self) -> usize {
        self.sessions
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_add_and_trim() {
        let mut session = ConversationSession::new("s1".into(), "docs".into(), 100);
        // Each char = ~0.25 tokens, so 400 chars ~ 100 tokens
        session.add_message(MessageRole::User, "a".repeat(200), Some(50));
        assert_eq!(session.messages.len(), 1);

        session.add_message(MessageRole::Assistant, "b".repeat(200), Some(60));
        // Total 110 > 100, should trim oldest
        assert_eq!(session.messages.len(), 1);
        assert_eq!(session.messages[0].content, "b".repeat(200));
    }

    #[test]
    fn session_manager_crud() {
        let mgr = SessionManager::with_defaults();
        let _session = mgr.create_session("s1".into(), "docs".into());
        assert!(mgr.get_session("s1").is_some());
        assert_eq!(mgr.session_count(), 1);
        mgr.delete_session("s1");
        assert_eq!(mgr.session_count(), 0);
    }

    #[test]
    fn cleanup_removes_expired() {
        let mgr = SessionManager::new(8192, Duration::from_millis(100));
        let mut session = mgr.create_session("s1".into(), "docs".into());
        session.updated_at_ms = now_ms() - 200;
        mgr.update_session(session);

        std::thread::sleep(Duration::from_millis(50));
        let removed = mgr.cleanup_expired();
        assert_eq!(removed, 1);
    }
}
