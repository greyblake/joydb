use std::time::Duration;

use axum::{
    Router,
    extract::{Form, Path, State},
    response::{Html, IntoResponse},
    routing::{get, post},
};
use joydb::{Joydb, JoydbConfig, JoydbMode, Model, SyncPolicy, adapters::JsonAdapter};
use maud::{Markup, PreEscaped, html};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// --- STORAGE ---

const DATA_PATH: &str = "data.json";

#[derive(Debug, Serialize, Deserialize, Clone, Model)]
struct Todo {
    id: Uuid,
    name: String,
    completed: bool,
}

#[derive(Debug, Deserialize)]
struct NewTodo {
    name: String,
}

joydb::define_state!(
    DbState,
    models: [Todo],
);

type Db = Joydb<DbState, JsonAdapter>;

#[tokio::main]
async fn main() {
    let config = JoydbConfig {
        mode: JoydbMode::Persistent {
            adapter: JsonAdapter::new(DATA_PATH, false),
            sync_policy: SyncPolicy::Periodic(Duration::from_secs(5)),
        },
    };
    let db = Db::open_with_config(config).unwrap();

    // Create an Axum router with routes
    let app = Router::new()
        .route("/", get(index))
        .route("/todos", post(add_todo))
        .route("/todos/{id}/toggle", post(toggle_todo))
        .with_state(db.clone());

    // Start the server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3001")
        .await
        .unwrap();

    println!(
        "Server running at http://{}",
        listener.local_addr().unwrap()
    );

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal(db))
        .await
        .unwrap();
}

// --- HANDLERS ---

// Handler for the index page
async fn index(State(db): State<Db>) -> impl IntoResponse {
    let pending_todos = db.get_all_by(|t: &Todo| !t.completed).unwrap();
    let completed_todos = db.get_all_by(|t: &Todo| t.completed).unwrap();

    Html(render_page(&pending_todos, &completed_todos).into_string())
}

// Handler for adding a new todo
async fn add_todo(State(db): State<Db>, Form(new_todo): Form<NewTodo>) -> impl IntoResponse {
    let new_todo = Todo {
        id: Uuid::new_v4(),
        name: new_todo.name,
        completed: false,
    };
    db.insert(&new_todo).unwrap();
    axum::response::Redirect::to("/")
}

// Handler for toggling todo completion status
async fn toggle_todo(State(db): State<Db>, Path(id): Path<Uuid>) -> impl IntoResponse {
    if let Some(mut todo) = db.get::<Todo>(&id).unwrap() {
        todo.completed = !todo.completed;
        db.update(todo).unwrap()
    }

    axum::response::Redirect::to("/")
}

// Shutdown signal handler
async fn shutdown_signal(_db: Db) {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for Ctrl+C signal");
}

// --- TEMPLATES ---

fn render_page(pending_todos: &[Todo], completed_todos: &[Todo]) -> Markup {
    html! {
        (PreEscaped("<!DOCTYPE html>"))
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { "Todo App" }
                script src="https://cdn.tailwindcss.com" {}
            }
            body class="bg-gray-100" {
                div class="max-w-md mx-auto p-6" {
                    h1 class="text-2xl font-bold mb-6 text-center" { "Todo App" }

                    // Add todo form
                    form class="mb-8" method="POST" action="/todos" {
                        div class="flex gap-2" {
                            input
                                type="text"
                                name="name"
                                placeholder="What needs to be done?"
                                class="flex-1 px-4 py-2 border rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500";
                            button
                                type="submit"
                                class="px-4 py-2 bg-blue-500 text-white rounded-lg hover:bg-blue-600 focus:outline-none focus:ring-2 focus:ring-blue-500" {
                                "Add"
                            }
                        }
                    }

                    // Pending todos section
                    h2 class="text-xl font-semibold mb-2" { "Pending" }
                    ul class="mb-6 space-y-2" {
                        @for todo in pending_todos {
                            li class="flex items-center justify-between p-3 bg-white rounded-lg shadow" {
                                span { (todo.name) }
                                form method="POST" action=(format!("/todos/{}/toggle", todo.id)) {
                                    button
                                        type="submit"
                                        class="px-3 py-1 bg-green-500 text-white rounded hover:bg-green-600" {
                                        "Complete"
                                    }
                                }
                            }
                        }
                        @if pending_todos.is_empty() {
                            li class="text-gray-500 italic" { "No pending tasks" }
                        }
                    }

                    // Completed todos section
                    h2 class="text-xl font-semibold mb-2" { "Completed" }
                    ul class="space-y-2" {
                        @for todo in completed_todos {
                            li class="flex items-center justify-between p-3 bg-white rounded-lg shadow opacity-75" {
                                span class="line-through" { (todo.name) }
                                form method="POST" action=(format!("/todos/{}/toggle", todo.id)) {
                                    button
                                        type="submit"
                                        class="px-3 py-1 bg-gray-500 text-white rounded hover:bg-gray-600" {
                                        "Undo"
                                    }
                                }
                            }
                        }
                        @if completed_todos.is_empty() {
                            li class="text-gray-500 italic" { "No completed tasks" }
                        }
                    }
                }
            }
        }
    }
}
