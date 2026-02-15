use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;
use hi_core::session::ChatSession;
use shared::config::ModelConfig;
use tokio::sync::Mutex;
use tokio::time::Instant;
use tracing::debug;

struct SessionEntry {
    session: Arc<Mutex<ChatSession>>,
    last_activity: Instant,
}

pub struct SessionManager {
    sessions: Mutex<HashMap<i64, SessionEntry>>,
    config: ModelConfig,
}

impl SessionManager {
    pub fn new(config: ModelConfig) -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
            config,
        }
    }

    fn session_config(&self) -> (u64, usize) {
        match self.config.remote.as_ref().and_then(|r| r.session.as_ref()) {
            Some(sc) => (sc.ttl_secs, sc.max_sessions),
            None => (3600, 100),
        }
    }

    pub async fn get_or_create(&self, chat_id: i64) -> Result<Arc<Mutex<ChatSession>>> {
        let mut sessions = self.sessions.lock().await;
        let (ttl_secs, max_sessions) = self.session_config();
        let now = Instant::now();

        // Sweep expired sessions
        let expired: Vec<i64> = sessions
            .iter()
            .filter(|(_, entry)| now.duration_since(entry.last_activity).as_secs() > ttl_secs)
            .map(|(id, _)| *id)
            .collect();

        for id in &expired {
            sessions.remove(id);
            debug!(
                chat_id = id,
                ttl_secs,
                "Evicted idle session"
            );
        }

        // Reuse existing session
        if let Some(entry) = sessions.get_mut(&chat_id) {
            entry.last_activity = now;
            debug!(chat_id, "Reused session");
            return Ok(Arc::clone(&entry.session));
        }

        // Capacity eviction: evict oldest-idle session if at limit
        if sessions.len() >= max_sessions {
            if let Some((&oldest_id, _)) = sessions
                .iter()
                .min_by_key(|(_, entry)| entry.last_activity)
            {
                sessions.remove(&oldest_id);
                debug!(
                    chat_id = oldest_id,
                    max_sessions,
                    "Evicted oldest session (capacity)"
                );
            }
        }

        // Create new session
        let session = ChatSession::new(self.config.clone())?;
        let session = Arc::new(Mutex::new(session));
        sessions.insert(
            chat_id,
            SessionEntry {
                session: Arc::clone(&session),
                last_activity: now,
            },
        );
        debug!(chat_id, "Created session");

        Ok(session)
    }

    #[allow(dead_code)]
    pub async fn session_count(&self) -> usize {
        self.sessions.lock().await.len()
    }

    pub async fn reset_session(&self, chat_id: i64) -> Result<bool> {
        let sessions = self.sessions.lock().await;
        match sessions.get(&chat_id) {
            Some(entry) => {
                let mut session = entry.session.lock().await;
                session.reset()?;
                Ok(true)
            }
            None => Ok(false),
        }
    }

    pub async fn compact_session(&self, chat_id: i64) -> Result<bool> {
        let sessions = self.sessions.lock().await;
        match sessions.get(&chat_id) {
            Some(entry) => {
                let mut session = entry.session.lock().await;
                Ok(session.run_compact().await)
            }
            None => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> ModelConfig {
        let json = r#"{
            "provider": "ollama",
            "model": "qwen2.5:14b",
            "context_window": 32000
        }"#;
        serde_json::from_str(json).unwrap()
    }

    fn test_config_with_session(ttl_secs: u64, max_sessions: usize) -> ModelConfig {
        let json = format!(
            r#"{{
            "provider": "ollama",
            "model": "qwen2.5:14b",
            "context_window": 32000,
            "remote": {{
                "session": {{
                    "ttl_secs": {ttl_secs},
                    "max_sessions": {max_sessions}
                }}
            }}
        }}"#
        );
        serde_json::from_str(&json).unwrap()
    }

    #[test]
    fn test_session_manager_creation() {
        let _manager = SessionManager::new(test_config());
    }

    #[tokio::test]
    async fn test_same_chat_id_returns_same_session() {
        let manager = SessionManager::new(test_config());
        let s1 = manager.get_or_create(100).await.unwrap();
        let s2 = manager.get_or_create(100).await.unwrap();
        assert!(Arc::ptr_eq(&s1, &s2));
        assert_eq!(manager.session_count().await, 1);
    }

    #[tokio::test]
    async fn test_different_chat_ids_get_different_sessions() {
        let manager = SessionManager::new(test_config());
        let s1 = manager.get_or_create(100).await.unwrap();
        let s2 = manager.get_or_create(200).await.unwrap();
        assert!(!Arc::ptr_eq(&s1, &s2));
        assert_eq!(manager.session_count().await, 2);
    }

    #[tokio::test]
    async fn test_concurrent_access_different_chat_ids() {
        let manager = Arc::new(SessionManager::new(test_config()));

        let m1 = Arc::clone(&manager);
        let m2 = Arc::clone(&manager);

        let (s1, s2) = tokio::join!(
            async move { m1.get_or_create(100).await.unwrap() },
            async move { m2.get_or_create(200).await.unwrap() },
        );

        assert!(!Arc::ptr_eq(&s1, &s2));
        assert_eq!(manager.session_count().await, 2);
    }

    #[tokio::test]
    async fn test_session_count_tracks_unique_chats() {
        let manager = SessionManager::new(test_config());
        assert_eq!(manager.session_count().await, 0);

        manager.get_or_create(1).await.unwrap();
        assert_eq!(manager.session_count().await, 1);

        manager.get_or_create(2).await.unwrap();
        assert_eq!(manager.session_count().await, 2);

        manager.get_or_create(1).await.unwrap();
        assert_eq!(manager.session_count().await, 2);
    }

    #[tokio::test(start_paused = true)]
    async fn test_ttl_eviction() {
        let manager = SessionManager::new(test_config_with_session(60, 100));

        manager.get_or_create(100).await.unwrap();
        assert_eq!(manager.session_count().await, 1);

        // Advance past TTL
        tokio::time::advance(std::time::Duration::from_secs(61)).await;

        // Next access should evict the expired session
        manager.get_or_create(200).await.unwrap();
        // chat 100 evicted, only chat 200 remains
        assert_eq!(manager.session_count().await, 1);
    }

    #[tokio::test]
    async fn test_capacity_eviction() {
        let manager = SessionManager::new(test_config_with_session(3600, 2));

        manager.get_or_create(100).await.unwrap();
        manager.get_or_create(200).await.unwrap();
        assert_eq!(manager.session_count().await, 2);

        // Creating a 3rd session should evict the oldest-idle (chat 100)
        manager.get_or_create(300).await.unwrap();
        assert_eq!(manager.session_count().await, 2);

        // Chat 100 should be gone
        let sessions = manager.sessions.lock().await;
        assert!(!sessions.contains_key(&100));
        assert!(sessions.contains_key(&200));
        assert!(sessions.contains_key(&300));
    }

    #[tokio::test(start_paused = true)]
    async fn test_session_reuse_updates_activity() {
        let manager = SessionManager::new(test_config_with_session(60, 100));

        manager.get_or_create(100).await.unwrap();

        // Advance 30s (within TTL)
        tokio::time::advance(std::time::Duration::from_secs(30)).await;

        // Reuse refreshes activity
        manager.get_or_create(100).await.unwrap();

        // Advance another 40s — total 70s since creation, but only 40s since last reuse
        tokio::time::advance(std::time::Duration::from_secs(40)).await;

        // Session should still be alive (40s < 60s TTL)
        manager.get_or_create(200).await.unwrap();
        assert_eq!(manager.session_count().await, 2);
    }

    #[tokio::test(start_paused = true)]
    async fn test_ttl_eviction_does_not_evict_fresh_sessions() {
        let manager = SessionManager::new(test_config_with_session(60, 100));

        manager.get_or_create(100).await.unwrap();
        manager.get_or_create(200).await.unwrap();

        // Advance 30s — both sessions still fresh
        tokio::time::advance(std::time::Duration::from_secs(30)).await;

        manager.get_or_create(300).await.unwrap();
        assert_eq!(manager.session_count().await, 3);
    }
}
