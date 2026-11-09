use crate::{
    JsonStore,
    models::{CreateTodo, Todo, UpdateTodo},
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::Html,
};

pub async fn health() -> &'static str {
    "ok"
}

pub async fn index() -> Html<&'static str> {
    Html(include_str!("../static/index.html"))
}

pub async fn list_todos(State(store): State<JsonStore>) -> Json<Vec<Todo>> {
    Json(store.all())
}

pub async fn get_todo(
    State(store): State<JsonStore>,
    Path(id): Path<u64>,
) -> Result<Json<Todo>, StatusCode> {
    store.get(id).map(Json).ok_or(StatusCode::NOT_FOUND)
}

pub async fn create_todo(
    State(store): State<JsonStore>,
    Json(payload): Json<CreateTodo>,
) -> Result<(StatusCode, Json<Todo>), StatusCode> {
    match store.create(payload.title) {
        Ok(todo) => Ok((StatusCode::CREATED, Json(todo))),
        Err(_) => Err(StatusCode::BAD_REQUEST),
    }
}

pub async fn update_todo(
    State(store): State<JsonStore>,
    Path(id): Path<u64>,
    Json(payload): Json<UpdateTodo>,
) -> Result<Json<Todo>, StatusCode> {
    match store.update(id, payload) {
        Ok(Some(todo)) => Ok(Json(todo)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::BAD_REQUEST),
    }
}

pub async fn delete_todo(State(store): State<JsonStore>, Path(id): Path<u64>) -> StatusCode {
    match store.delete(id) {
        Ok(true) => StatusCode::NO_CONTENT,
        _ => StatusCode::NOT_FOUND,
    }
}
