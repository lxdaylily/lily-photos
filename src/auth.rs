use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

use uuid::Uuid;

#[derive(Clone)]
pub struct AuthState {
    admin_password: Arc<Mutex<Option<String>>>,
    sessions: Arc<Mutex<HashSet<String>>>,
}

impl AuthState {
    pub fn new(admin_password: Option<String>) -> Self {
        Self {
            admin_password: Arc::new(Mutex::new(admin_password)),
            sessions: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    pub fn requires_setup(&self) -> bool {
        self.admin_password
            .lock()
            .ok()
            .map(|password| password.is_none())
            .unwrap_or(true)
    }

    pub fn verify_password(&self, password: &str) -> bool {
        self.admin_password
            .lock()
            .ok()
            .and_then(|stored| stored.clone())
            .map(|stored| password == stored)
            .unwrap_or(false)
    }

    pub fn set_password_if_unset(&self, password: String) -> Result<bool, String> {
        let mut stored = self
            .admin_password
            .lock()
            .map_err(|_| "auth lock poisoned")?;

        if stored.is_some() {
            return Ok(false);
        }

        *stored = Some(password);
        Ok(true)
    }

    pub fn create_session(&self) -> Result<String, String> {
        let token = Uuid::new_v4().to_string();
        let mut sessions = self.sessions.lock().map_err(|_| "auth lock poisoned")?;
        sessions.insert(token.clone());
        Ok(token)
    }

    pub fn revoke_session(&self, token: &str) -> Result<(), String> {
        let mut sessions = self.sessions.lock().map_err(|_| "auth lock poisoned")?;
        sessions.remove(token);
        Ok(())
    }

    pub fn is_authenticated(&self, token: Option<&str>) -> bool {
        let Some(token) = token else {
            return false;
        };

        self.sessions
            .lock()
            .ok()
            .map(|sessions| sessions.contains(token))
            .unwrap_or(false)
    }
}
