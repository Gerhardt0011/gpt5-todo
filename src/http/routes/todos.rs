use axum::{extract::State, routing::{get, post}, Router, Json};
use axum::http::StatusCode;
use serde::Deserialize;

use crate::{application::todo_service::TodoService, domain::todo::{CreateTodo, TodoId, UpdateTodo}};

#[derive(Clone)]
pub struct AppState<S: TodoService> { pub service: S }

pub fn router<S: TodoService + Clone + Send + Sync + 'static>(state: AppState<S>) -> Router {
    Router::new()
        .route("/todos", post(create_todo::<S>).get(list_todos::<S>))
        .route("/todos/:id", get(get_todo::<S>).put(update_todo::<S>).delete(delete_todo::<S>))
        .with_state(state)
}

async fn create_todo<S: TodoService>(State(state): State<AppState<S>>, Json(payload): Json<CreateTodo>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let todo = state.service.create(payload).await.map_err(internal_error)?;
    Ok(Json(serde_json::json!({ "id": todo.id.0, "title": todo.title, "description": todo.description, "status": format_status(&todo), "created_at": todo.created_at, "updated_at": todo.updated_at })))
}

async fn list_todos<S: TodoService>(State(state): State<AppState<S>>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let todos = state.service.list().await.map_err(internal_error)?;
    Ok(Json(serde_json::json!({ "items": todos.into_iter().map(|t| serde_json::json!({
        "id": t.id.0,
        "title": t.title,
        "description": t.description,
        "status": format_status(&t),
        "created_at": t.created_at,
        "updated_at": t.updated_at,
    })).collect::<Vec<_>>() })))
}

async fn get_todo<S: TodoService>(State(state): State<AppState<S>>, axum::extract::Path(id): axum::extract::Path<String>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let id = parse_id(&id)?;
    let todo = state.service.get(id).await.map_err(internal_error)?;
    match todo {
        Some(t) => Ok(Json(serde_json::json!({ "id": t.id.0, "title": t.title, "description": t.description, "status": format_status(&t), "created_at": t.created_at, "updated_at": t.updated_at }))),
        None => Err((StatusCode::NOT_FOUND, "Not found".into()))
    }
}

#[derive(Deserialize)]
struct UpdateBody { title: Option<String>, description: Option<String>, status: Option<String> }

async fn update_todo<S: TodoService>(State(state): State<AppState<S>>, axum::extract::Path(id): axum::extract::Path<String>, Json(payload): Json<UpdateBody>) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let id = parse_id(&id)?;
    let status = match payload.status.as_deref() {
        Some("pending") => Some(crate::domain::todo::TodoStatus::Pending),
        Some("done") => Some(crate::domain::todo::TodoStatus::Done),
        Some(_) => return Err((StatusCode::BAD_REQUEST, "invalid status".into())),
        None => None,
    };
    let updated = state.service.update(id, UpdateTodo { title: payload.title, description: payload.description, status }).await.map_err(internal_error)?;
    match updated {
        Some(t) => Ok(Json(serde_json::json!({ "id": t.id.0, "title": t.title, "description": t.description, "status": format_status(&t), "created_at": t.created_at, "updated_at": t.updated_at }))),
        None => Err((StatusCode::NOT_FOUND, "Not found".into()))
    }
}

async fn delete_todo<S: TodoService>(State(state): State<AppState<S>>, axum::extract::Path(id): axum::extract::Path<String>) -> Result<StatusCode, (StatusCode, String)> {
    let id = parse_id(&id)?;
    let deleted = state.service.delete(id).await.map_err(internal_error)?;
    if deleted { Ok(StatusCode::NO_CONTENT) } else { Err((StatusCode::NOT_FOUND, "Not found".into())) }
}

fn parse_id(s: &str) -> Result<TodoId, (StatusCode, String)> { uuid::Uuid::parse_str(s).map(|u| TodoId(u)).map_err(|_| (StatusCode::BAD_REQUEST, "invalid id".into())) }

fn format_status(t: &crate::domain::todo::Todo) -> &'static str { match t.status { crate::domain::todo::TodoStatus::Pending => "pending", crate::domain::todo::TodoStatus::Done => "done" } }

fn internal_error<E: std::fmt::Display>(e: E) -> (StatusCode, String) { (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e)) }
