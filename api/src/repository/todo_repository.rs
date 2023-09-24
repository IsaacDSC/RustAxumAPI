use crate::{
    model::TodoModel,
    schema::{TodoListDTO, UpdateTodoDto},
    AppState,
};
use sqlx::Error;
use std::sync::Arc;

pub struct TodoRepository {
    data: Arc<AppState>,
}

impl TodoRepository {
    pub fn new(data: Arc<AppState>) -> TodoRepository {
        TodoRepository { data }
    }

    pub async fn create(&mut self, input: TodoListDTO) -> Result<TodoModel, Error> {
        let result = sqlx::query_as!(
            TodoModel,
            "INSERT INTO todo (title,content,category) VALUES ($1, $2, $3) RETURNING *",
            input.title.to_string(),
            input.content.to_string(),
            input.category.to_owned().unwrap_or("".to_string())
        )
        .fetch_one(&self.data.db)
        .await;
        result
    }

    pub async fn find_all(&mut self, limit: usize, offset: usize) -> Result<Vec<TodoModel>, Error> {
        sqlx::query_as!(
            TodoModel,
            "SELECT * FROM todo ORDER by id LIMIT $1 OFFSET $2;",
            limit as i32,
            offset as i32
        )
        .fetch_all(&self.data.db)
        .await
    }

    pub async fn get_by_id(&mut self, id: uuid::Uuid) -> Result<Vec<TodoModel>, Error> {
        sqlx::query_as!(TodoModel, "SELECT * FROM todo WHERE id = $1;", id,)
            .fetch_all(&self.data.db)
            .await
    }

    pub async fn update(
        &mut self,
        id: uuid::Uuid,
        input: UpdateTodoDto,
    ) -> Result<TodoModel, Error> {
        let get_todo = sqlx::query_as!(TodoModel, "SELECT * FROM todo WHERE id = $1;", id)
            .fetch_one(&self.data.db)
            .await;
        if get_todo.is_err() {
            return get_todo;
        }
        let now = chrono::Utc::now();
        let todo = get_todo.unwrap();
        sqlx::query_as!(
            TodoModel,
            "UPDATE todo SET title = $1, content = $2, category = $3, published = $4, updated_at = $5 WHERE id = $6 RETURNING *",
            input.title.to_owned().unwrap_or(todo.title),
            input.content.to_owned().unwrap_or(todo.content),
            input.category.to_owned().unwrap_or(todo.category.unwrap()),
            input.published.unwrap_or(todo.published.unwrap()),
            now,
            id
        ).fetch_one(&self.data.db).await
    }

    pub async fn delete(&mut self, id: uuid::Uuid) -> Result<bool, Error> {
        let result = sqlx::query!("DELETE FROM todo WHERE id = $1", id)
            .execute(&self.data.db)
            .await;
        match result {
            Ok(data) => {
                let rows_affected = data.rows_affected();
                if rows_affected == 0 || rows_affected > 1 {
                    return Ok(false);
                }
                return Ok(true);
            }
            Err(err) => Err(err),
        }

    }
}
