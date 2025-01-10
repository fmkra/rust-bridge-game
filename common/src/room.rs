use std::sync::Arc;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct RoomId(Arc<str>);

impl RoomId {
    pub fn new(id: Arc<str>) -> Self {
        Self(id)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum Visibility {
    Public,
    Private,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RoomInfo {
    pub id: RoomId,
    pub visibility: Visibility,
}
