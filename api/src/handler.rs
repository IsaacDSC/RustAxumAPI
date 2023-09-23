use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;

use crate::{
    model::TodoModel,
    schema::{FilterOptions, TodoListDTO, UpdateTodoDto},
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
    let result = sqlx::query_as!(
        TodoModel,
        "INSERT INTO todo (title,content,category) VALUES ($1, $2, $3) RETURNING *",
        body.title.to_string(),
        body.content.to_string(),
        body.category.to_owned().unwrap_or("".to_string())
    )
    .fetch_one(&data.db)
    .await;

    match result {
        Ok(data) => {
            let note_response = json!({"status": "success","data": json!({
                "todo": data
            })});

            return Ok((StatusCode::CREATED, Json(note_response)));
        }
        Err(e) => {
            if e.to_string()
                .contains("duplicate key value violates unique constraint")
            {
                let error_response = serde_json::json!({
                    "status": "fail",
                    "message": "Note with that title already exists",
                });
                return Err((StatusCode::CONFLICT, Json(error_response)));
            }
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
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
    let result = sqlx::query_as!(
        TodoModel,
        "SELECT * FROM todo ORDER by id LIMIT $1 OFFSET $2;",
        limit as i32,
        offset as i32
    )
    .fetch_all(&data.db)
    .await;

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
    let result = sqlx::query_as!(TodoModel, "SELECT * FROM todo WHERE id = $1;", id,)
        .fetch_all(&data.db)
        .await;

    match result {
        Err(_) => {
            let response_error = serde_json::json!({
                "status": "fail",
                "message":format!("Note with ID: {} not found", id),
            });
            return Err((StatusCode::NOT_FOUND, Json(response_error)));
        }
        Ok(note) => {
            let note_response = serde_json::json!({"status": "success","data": serde_json::json!({
                "note": note
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
    let result_todo = sqlx::query_as!(TodoModel, "SELECT * FROM todo WHERE id = $1;", id)
        .fetch_one(&data.db)
        .await;
    if result_todo.is_err() {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": format!("Note with ID: {} not found", id)
        });
        return Err((StatusCode::NOT_FOUND, Json(error_response)));
    }
    let todo = result_todo.unwrap();
    let now = chrono::Utc::now();
    let result=sqlx::query_as!(
        TodoModel,
        "UPDATE todo SET title = $1, content = $2, category = $3, published = $4, updated_at = $5 WHERE id = $6 RETURNING *",
        body.title.to_owned().unwrap_or(todo.title),
        body.content.to_owned().unwrap_or(todo.content),
        body.category.to_owned().unwrap_or(todo.category.unwrap()),
        body.published.unwrap_or(todo.published.unwrap()),
        now,
        id
    ).fetch_one(&data.db).await;
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
    let rows_affected = sqlx::query!("DELETE FROM todo WHERE id = $1", id)
        .execute(&data.db)
        .await
        .unwrap()
        .rows_affected();
    if rows_affected == 0 || rows_affected > 1 {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": format!("Note with ID: {} not found", id)
        });
        return Err((StatusCode::NOT_FOUND, Json(error_response)));
    }
    Ok(StatusCode::NO_CONTENT)
}
