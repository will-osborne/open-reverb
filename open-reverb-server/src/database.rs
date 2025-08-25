// This is a placeholder for a real database implementation.
// In a production application, you would use a proper database like PostgreSQL, SQLite, etc.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::auth::Credentials;

// Simple in-memory database for demonstration purposes
pub struct Database {
    users: HashMap<String, Credentials>,
    user_ids: HashMap<String, Uuid>,
}

impl Database {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
            user_ids: HashMap::new(),
        }
    }
    
    pub fn add_user(&mut self, username: &str, password_hash: &str, user_id: Uuid) -> bool {
        if self.users.contains_key(username) {
            return false;
        }
        
        self.users.insert(
            username.to_string(),
            Credentials {
                username: username.to_string(),
                password_hash: password_hash.to_string(),
            },
        );
        
        self.user_ids.insert(username.to_string(), user_id);
        
        true
    }
    
    pub fn get_user(&self, username: &str) -> Option<&Credentials> {
        self.users.get(username)
    }
    
    pub fn get_user_id(&self, username: &str) -> Option<Uuid> {
        self.user_ids.get(username).copied()
    }
}

// Global database instance
lazy_static::lazy_static! {
    static ref DB: Arc<Mutex<Database>> = Arc::new(Mutex::new(Database::new()));
}

pub fn get_db() -> Arc<Mutex<Database>> {
    DB.clone()
}