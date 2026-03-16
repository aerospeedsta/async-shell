use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use anyhow::Result;
use ed25519_dalek::SigningKey;
use rand_core::OsRng;

use crate::pty::PtySession;

pub struct SessionManager {
    sessions: Arc<Mutex<HashMap<String, Arc<PtySession>>>>,
    registered_agents: Arc<Mutex<HashSet<String>>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            registered_agents: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    /// Registers a new agent session, generating a unique Ed25519 public key as their ID.
    pub fn register_agent(&self) -> String {
        let mut csprng = OsRng;
        let signing_key: SigningKey = SigningKey::generate(&mut csprng);
        let verifying_key = signing_key.verifying_key();
        let agent_id = hex::encode(verifying_key.as_bytes());
        
        self.registered_agents.lock().unwrap().insert(agent_id.clone());
        agent_id
    }

    /// Spawn a session, validating the agent_id
    pub fn spawn(&self, id: &str, agent_id: &str, command: &str) -> Result<()> {
        // Enforce Authentication
        {
            let agents = self.registered_agents.lock().unwrap();
            if !agents.contains(agent_id) {
                anyhow::bail!("Authentication failed: Invalid or unregistered agent_id");
            }
        }

        let session = PtySession::spawn(id, command)?;
        self.sessions.lock().unwrap().insert(id.to_string(), Arc::new(session));
        Ok(())
    }

    /// Write to stdin, validating the agent_id exists
    pub fn write_stdin(&self, id: &str, agent_id: &str, input: &str) -> Result<()> {
        // Enforce Authentication
        {
            let agents = self.registered_agents.lock().unwrap();
            if !agents.contains(agent_id) {
                anyhow::bail!("Authentication failed: Invalid or unregistered agent_id");
            }
        }

        let sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get(id) {
            session.write_stdin(input)
        } else {
            anyhow::bail!("Session {} not found", id)
        }
    }

    /// Read history, validating the agent_id exists in the registry to prevent unauthorized tampering
    pub fn read_history(&self, id: &str, agent_id: &str, start_line: Option<usize>, max_lines: Option<usize>) -> Result<String> {
        // Enforce Authentication
        {
            let agents = self.registered_agents.lock().unwrap();
            if !agents.contains(agent_id) {
                anyhow::bail!("Authentication failed: Invalid or unregistered agent_id");
            }
        }

        let sessions = self.sessions.lock().unwrap();
        if let Some(session) = sessions.get(id) {
            Ok(session.read_history(agent_id, start_line, max_lines))
        } else {
            anyhow::bail!("Session {} not found", id)
        }
    }

    pub fn list_sessions(&self) -> Vec<String> {
        let sessions = self.sessions.lock().unwrap();
        sessions.keys().cloned().collect()
    }

    pub fn kill(&self, id: &str) -> Result<()> {
        let mut sessions = self.sessions.lock().unwrap();
        if sessions.remove(id).is_some() {
            // Drop handles to let process eventually terminate / SIGKILL logic could go here
            Ok(())
        } else {
            anyhow::bail!("Session {} not found", id)
        }
    }
}
