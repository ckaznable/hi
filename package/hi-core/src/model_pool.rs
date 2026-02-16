use crate::provider::{ChatAgent, create_agent_from_small};
use anyhow::Result;
use shared::config::SmallModelConfig;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, Weak};

#[derive(Hash, Eq, PartialEq, Clone)]
struct AgentKey {
    provider: String,
    model: String,
}

pub struct ModelPool {
    agents: Mutex<HashMap<AgentKey, Weak<ChatAgent>>>,
}

impl ModelPool {
    pub fn new() -> Self {
        Self {
            agents: Mutex::new(HashMap::new()),
        }
    }

    pub fn get_or_create(
        &self,
        config: &SmallModelConfig,
        preamble: Option<&str>,
    ) -> Result<Arc<ChatAgent>> {
        let key = AgentKey {
            provider: format!("{:?}", config.provider),
            model: config.model.clone(),
        };

        let mut agents = self.agents.lock().unwrap();

        if let Some(weak) = agents.get(&key) {
            if let Some(arc) = weak.upgrade() {
                return Ok(arc);
            }
        }

        let agent = create_agent_from_small(config, preamble)?;
        let arc = Arc::new(agent);
        agents.insert(key, Arc::downgrade(&arc));
        Ok(arc)
    }
}
