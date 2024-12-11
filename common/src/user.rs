use std::sync::Arc;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct User {
    username: Arc<str>,
}

impl User {
    pub fn new(username: &str) -> Self {
        Self {
            username: username.into(),
        }
    }

    pub fn get_username(&self) -> &str {
        &self.username
    }
}
