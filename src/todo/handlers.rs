use axum::{
    extract::{Path, Request, State},
    Extension, Json, RequestExt,
};

use crate::{
    lib::{http_error::HttpError, repository::CrudRepository},
    AppState,
};

use super::model::{NewTodo, Todo};

pub async fn get_handler(State(state): State<AppState>) -> Result<Json<Vec<Todo>>, HttpError> {
    state
        .todo_repository
        .get_all()
        .await
        .map(Json)
        .map_err(Into::into)
}
pub async fn get_one_handler(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<Todo>, HttpError> {
    state
        .todo_repository
        .find_one(id)
        .await
        .map(Json)
        .map_err(Into::into)
}
pub async fn create_handler(
    State(state): State<AppState>,
    Json(new_todo): Json<NewTodo>,
) -> Result<Json<Todo>, HttpError> {
    state
        .todo_repository
        .create(new_todo)
        .await
        .map(Json)
        .map_err(Into::into)
}
