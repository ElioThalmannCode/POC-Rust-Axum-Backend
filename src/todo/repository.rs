use sqlx::{query_as, PgPool};

use crate::lib::repository::{CrudRepository, FindOneError, GetAllError};

use super::model::{NewTodo, Todo};

#[derive(Clone)]
pub struct TodoRepository {
    pool: PgPool,
}
impl TodoRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
impl CrudRepository<Todo, NewTodo> for TodoRepository {
    async fn get_all(&self) -> Result<Vec<Todo>, GetAllError> {
        query_as!(Todo, "select * from todos;")
            .fetch_all(&self.pool)
            .await
            .map_err(Into::into)
    }
    async fn find_one(&self, id: i32) -> Result<Todo, FindOneError> {
        query_as!(Todo, "SELECT * FROM todos WHERE id = $1 LIMIT 1;", id)
            .fetch_one(&self.pool)
            .await
            .map_err(Into::into)
    }
    async fn create(&self, item: NewTodo) -> Result<Todo, crate::lib::repository::CreateError> {
        query_as!(
            Todo,
            "INSERT INTO todos (task) VALUES ($1) RETURNING id, task;",
            item.task
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Into::into)
    }
}
