use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;

use crate::{
    repository::todo_repository::TodoRepository,
    schema::{FilterOptions, TodoListDTO, UpdateTodoDto},
    service::todo_service::TodoService,
    AppState,
};

pub async fn health_checker_handler() -> impl IntoResponse {
    const MESSAGE: &str = "Simple CRUD API with Rust, SQLX, Postgres,and Axum";

    let json_response = serde_json::json!({
        "status": "success",
        "message": MESSAGE
    });

    Json(json_response)
}

pub async fn create_todo_handler(
    State(data): State<Arc<AppState>>,
    Json(body): Json<TodoListDTO>,
) -> impl IntoResponse {
    let repository = TodoRepository::new(data); //TODO: inject repository
    let mut service = TodoService::new(repository);//TODO: inject service
    let result = service.create_note(body).await;
    match result {
        Ok(data) => {
            let note_response = json!({"status": "success","data": json!({
                "todo": data
            })});

            return Ok((StatusCode::CREATED, Json(note_response)));
        }
        Err(e) => {
            let status_error = if e.to_string() == "Todo with that title already exists" {
                StatusCode::CONFLICT
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            return Err((
                status_error,
                Json(json!({"status": "error","message": format!("{:?}", e)})),
            ));
        }
    }
}

pub async fn findall_todo_handler(
    opts: Option<Query<FilterOptions>>,
    State(data): State<Arc<AppState>>,
) -> impl IntoResponse {
    let Query(opts) = opts.unwrap_or_default();
    let limit = opts.limit.unwrap_or(10);
    let offset = (opts.page.unwrap_or(1) - 1) * limit;
    let repository = TodoRepository::new(data); //TODO: inject repository
    let mut service = TodoService::new(repository);//TODO: inject service
    let result = service.find_all_note(limit, offset).await;
    match result {
        Ok(todo) => {
            let json_response = serde_json::json!({
                "status": "success",
                "results": todo.len(),
                "todo": todo
            });
            Ok(Json(json_response))
        }
        Err(e) => {
            println!("{}", e);
            let error_response = serde_json::json!({
                "status": "fail",
                "message": "Something bad happened while fetching all note items",
            });
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)));
        }
    }
}

pub async fn get_by_id_todo_handler(
    Path(id): Path<uuid::Uuid>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let repository = TodoRepository::new(data); //TODO: inject repository
    let mut service = TodoService::new(repository);//TODO: inject service
    let result = service.get_note(id).await;
    match result {
        Err(err) => {
            let response_error = serde_json::json!({
                "status": "fail",
                "message": err.to_string(),
            });
            return Err((StatusCode::NOT_FOUND, Json(response_error)));
        }
        Ok(data) => {
            let note_response = serde_json::json!({"status": "success","data": serde_json::json!({
                "todo": data
            })});
            return Ok(Json(note_response));
        }
    }
}

pub async fn update_todo_handler(
    Path(id): Path<uuid::Uuid>,
    State(data): State<Arc<AppState>>,
    Json(body): Json<UpdateTodoDto>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let repository = TodoRepository::new(data); //TODO: inject repository
    let mut service = TodoService::new(repository);//TODO: inject service
    let result = service.update_note(id, body).await;
    match result {
        Ok(note) => {
            let note_response = serde_json::json!({"status": "success","data": serde_json::json!({
                "note": note
            })});

            return Ok(Json(note_response));
        }
        Err(err) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"status": "error","message": format!("{:?}", err)})),
            ));
        }
    }
}

pub async fn delete_todo_handler(
    Path(id): Path<uuid::Uuid>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let repository = TodoRepository::new(data); //TODO: inject repository
    let mut service = TodoService::new(repository);
    let deleted = service.delete_note(id).await;
    match deleted {
        Some(err) => {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": err.to_string()
            });
            return Err((StatusCode::NOT_FOUND, Json(error_response)))
        }
        None=>{
            Ok(StatusCode::NO_CONTENT)
        }
    }
}
