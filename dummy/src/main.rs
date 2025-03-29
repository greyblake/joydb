use uuid::Uuid;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct UserId(Uuid);

#[derive(Debug, Clone, Serialize, Deserialize)]
struct User {
    id: UserId,
    email: String,
}

fn main() {
    println!("Hello, world!");
}
