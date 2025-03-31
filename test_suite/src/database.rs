use serde::{Deserialize, Serialize};
use toydb::{Model, Toydb, define_state};
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
}

define_state! {
    DatabaseState,
    models: [User, Post],
}

pub type Database = Toydb<DatabaseState>;
