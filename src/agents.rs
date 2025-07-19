use crate::llm::LLMClient;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::task::JoinHandle;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub enum AgentStatus {
    Pending,
    Running,
    Completed,
    Failed(String),
}

#[derive(Debug)]
pub struct Agent {
    pub id: Uuid,
    pub name: String,
    pub status: AgentStatus,
    result: Arc<Mutex<Option<String>>>,
    handle: Option<JoinHandle<()>>,
}

impl Agent {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            status: AgentStatus::Pending,
            result: Arc::new(Mutex::new(None)),
            handle: None,
        }
    }

    pub fn result(&self) -> Option<String> {
        self.result.lock().unwrap().clone()
    }
}

pub struct AgentManager {
    agents: HashMap<Uuid, Agent>,
}

impl AgentManager {
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
        }
    }

    pub fn spawn_llm_agent(
        &mut self,
        name: impl Into<String>,
        client: Arc<dyn LLMClient>,
        messages: Vec<crate::models::Message>,
        model: String,
    ) -> Uuid {
        let mut agent = Agent::new(name);
        let agent_id = agent.id;
        agent.status = AgentStatus::Running;
        let result_handle = agent.result.clone();
        let handle = tokio::spawn(async move {
            let resp = client.send_message(&messages, &model).await;
            let mut lock = result_handle.lock().unwrap();
            *lock = Some(match resp {
                Ok(r) => r,
                Err(e) => format!("Error: {}", e),
            });
        });
        agent.handle = Some(handle);
        self.agents.insert(agent_id, agent);
        agent_id
    }

    pub fn spawn_command_agent<F>(&mut self, name: impl Into<String>, fut: F) -> Uuid
    where
        F: std::future::Future<Output = Result<()>> + Send + 'static,
    {
        let mut agent = Agent::new(name);
        let id = agent.id;
        agent.status = AgentStatus::Running;
        let handle = tokio::spawn(async move {
            let _ = fut.await;
        });
        agent.handle = Some(handle);
        self.agents.insert(id, agent);
        id
    }

    pub fn cleanup_finished(&mut self) {
        for agent in self.agents.values_mut() {
            if let Some(handle) = agent.handle.as_mut() {
                if handle.is_finished() {
                    agent.status = AgentStatus::Completed;
                    // Remove handle to avoid polling again
                    let _ = agent.handle.take();
                }
            }
        }
    }

    pub fn agent_status(&self, id: &Uuid) -> Option<AgentStatus> {
        self.agents.get(id).map(|a| a.status.clone())
    }

    pub fn agent_result(&self, id: &Uuid) -> Option<Option<String>> {
        self.agents.get(id).map(|a| a.result())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyClient;
    #[async_trait::async_trait]
    impl LLMClient for DummyClient {
        async fn send_message(
            &self,
            _m: &[crate::models::Message],
            _model: &str,
        ) -> Result<String> {
            Ok("ok".into())
        }
        fn provider(&self) -> crate::models::LLMProvider {
            crate::models::LLMProvider::OpenAI
        }
    }

    #[tokio::test]
    async fn test_spawn_llm_agent() {
        let mut mgr = AgentManager::new();
        let client = Arc::new(DummyClient);
        let id = mgr.spawn_llm_agent("test", client, vec![], "model".into());
        // Allow spawned task to complete
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        mgr.cleanup_finished();
        assert_eq!(mgr.agent_status(&id), Some(AgentStatus::Completed));
        assert_eq!(mgr.agent_result(&id), Some(Some("ok".into())));
    }
}
