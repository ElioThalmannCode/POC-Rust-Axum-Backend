use axum::{
    body::Body,
    http::{Response, StatusCode},
    response::IntoResponse,
};
use serde_json::json;

pub struct HttpError {
    msg: String,
    code: u16,
}

impl HttpError {
    pub fn internal_server_error() -> HttpError {
        Self {
            msg: "something went wrong, try again later".to_string(),
            code: 500,
        }
    }
    pub fn not_found_error(msg: String) -> HttpError {
        Self { msg, code: 404 }
    }
    pub fn unauthorized(msg: String) -> HttpError {
        Self { msg, code: 401 }
    }
    pub fn conflict(msg: String) -> HttpError {
        Self { msg, code: 409 }
    }
}

impl IntoResponse for HttpError {
    fn into_response(self) -> Response<Body> {
        create_error_response(self.code, self.msg)
    }
}
fn create_error_response(status: u16, msg: String) -> Response<Body> {
    Response::builder()
        .status(StatusCode::from_u16(status).unwrap())
        .header("content-type", "application/json")
        .body(Body::from(json!({"error":msg}).to_string()))
        .unwrap()
}
