use std::io::{Error, ErrorKind};

use crate::{
    model::TodoModel,
    repository::todo_repository,
    schema::{TodoListDTO, UpdateTodoDto},
};

pub struct TodoService {
    repository: todo_repository::TodoRepository, //TODO: impl traits <interface>
}

impl TodoService {
    pub fn new(repository: todo_repository::TodoRepository) -> TodoService {
        TodoService {
            repository: repository,
        }
    }

    pub async fn create_note(&mut self, input: TodoListDTO) -> Result<TodoModel, Error> {
        let result = self.repository.create(input).await;
        match result {
            Err(err) => {
                if err
                    .to_string()
                    .contains("duplicate key value violates unique constraint")
                {
                    let custom_error =
                        Error::new(ErrorKind::Other, "Todo with that title already exists");
                    return Err(custom_error);
                }
                println!("recovery_error: {}", err);
                let custom_error = Error::new(ErrorKind::Other, "Internal-Server-Error");
                Err(custom_error)
            }
            Ok(todo) => Ok(todo),
        }
    }

    pub async fn find_all_note(
        &mut self,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<TodoModel>, Error> {
        let result = self.repository.find_all(limit, offset).await;
        match result {
            Err(err) => {
                println!("recovery_error: {}", err);
                let custom_error = Error::new(ErrorKind::Other, "Internal-Server-Error");
                Err(custom_error)
            }
            Ok(data) => Ok(data),
        }
    }

    pub async fn get_note(&mut self, id: uuid::Uuid) -> Result<Vec<TodoModel>, Error> {
        let result = self.repository.get_by_id(id).await;
        match result {
            Err(err) => {
                println!("recovery_error: {}", err);
                let custom_error =
                    Error::new(ErrorKind::Other, format!("Note with ID: {} not found", id));
                Err(custom_error)
            }
            Ok(data) => Ok(data),
        }
    }

    pub async fn update_note(
        &mut self,
        id: uuid::Uuid,
        input: UpdateTodoDto,
    ) -> Result<TodoModel, Error> {
        let result = self.repository.update(id, input).await;
        match result {
            Err(err) => {
                println!("recovery_error: {}", err);
                let custom_error = Error::new(
                    ErrorKind::Other,
                    format!("Note with ID: {} not updated", id),
                );
                Err(custom_error)
            }
            Ok(data) => Ok(data),
        }
    }

    pub async fn delete_note(&mut self, id: uuid::Uuid) -> Option<Error> {
        let result = self.repository.delete(id).await;
        match result {
            Err(err) => {
                println!("recovery_error: {}", err);
                let custom_error = Error::new(
                    ErrorKind::Other,
                    format!("Note with ID: {} not deleted", id),
                );
                Some(custom_error)
            }
            Ok(is_deleted) => {
                if !is_deleted {
                    let custom_error = Error::new(
                        ErrorKind::Other,
                        format!("Note with ID: {} not deleted", id),
                    );
                    return Some(custom_error);
                }
                None
            }
        }
    }
}
