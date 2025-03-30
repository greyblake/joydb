use axum::{routing::get, Router, extract::State};
use toydb::{Model, define_storage};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use std::sync::{Arc, Mutex};

#[tokio::main]
async fn main() {
    let app_state = AppState::init();

    // Create an Axum router with one route
    let app = Router::new()
        .route("/", get(hello_world))
        .with_state(app_state.clone());

    // Start the server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("Server running at http://{}", listener.local_addr().unwrap());

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(app_state.clone()))
        .await
        .unwrap();
}

// Simple handler function returning a plain text response
async fn hello_world(
    State(AppState { db }): State<AppState>,
) -> String {
    let mut db = db.lock().unwrap();
    db.todos().insert(Todo {
        id: TodoId(Uuid::new_v4()),
        name: "Hello, World!".to_string(),
        completed: false,
    });


    let todos = db.todos().get_all();

    todos.iter().
        map(|todo|  {
            let Todo { id, name, completed } = todo;
            format!("{}: {} ({})", id.0, name, completed)
        }).collect::<Vec<_>>().join("\n")
}

#[derive(Clone)]
struct AppState {
    db: Arc<Mutex<Db>>,
}

impl AppState {
    fn init() -> Self {
        let db = Db::load_or_create(DB_PATH).unwrap();
        Self {
            db: Arc::new(Mutex::new(db)),
        }
    }
}


// When this function returns, Axum will gracefully shut down,
// and `Drop` implementations (if any) will execute normally.
// Which make DB to flush to disk
async fn shutdown_signal(app_state: AppState) {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for Ctrl+C signal");
}



// --- STORAGE ---

const DB_PATH: &str = "db.json";

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
struct TodoId(Uuid);

#[derive(Debug, Serialize, Deserialize, Clone, Model)]
struct Todo {
    id: TodoId,
    name: String,
    completed: bool,
}


define_storage! {
    Db,
    todos: Todo,
}
