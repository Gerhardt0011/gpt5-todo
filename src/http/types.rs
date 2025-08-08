use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError { pub message: String }

impl IntoResponse for ApiError {
    fn into_response(self) -> Response { (StatusCode::BAD_REQUEST, axum::Json(self)).into_response() }
}
