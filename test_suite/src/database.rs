use joydb::{Joydb, Model, adapters::JsonAdapter, state};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Model)]
pub struct Post {
    pub id: Uuid,
    pub content: String,
    pub user_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, Model)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub age: u8,
}

state! {
    DatabaseState,
    models: [User, Post],
}

pub type Database = Joydb<DatabaseState, JsonAdapter>;
