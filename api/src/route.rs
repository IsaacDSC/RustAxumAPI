use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
};

use crate::{
    handler::{
        create_todo_handler, delete_todo_handler, findall_todo_handler, get_by_id_todo_handler,
        health_checker_handler, update_todo_handler,
    },
    AppState,
};

pub fn start_routes(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/health", get(health_checker_handler))
        .route("/todos", get(findall_todo_handler))
        .route("/todo", post(create_todo_handler))
        .route(
            "/todo/:id",
            get(get_by_id_todo_handler)
                .patch(update_todo_handler)
                .delete(delete_todo_handler),
        )
        .with_state(app_state)
}
