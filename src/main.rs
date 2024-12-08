use std::{sync::Arc, time::Duration};

use ::sqlx::PgPool;
use auth::AuthRoutes;
use axum::{
    body::{Body, HttpBody},
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::{self, Next},
    response::Response,
    routing::{get, post},
    Router,
};
use lib::http_error::HttpError;
use redis::{aio::MultiplexedConnection, AsyncCommands, Commands, Connection};
mod auth;
mod lib;
mod todo;
use todo::handlers;
use todo::repository::TodoRepository;
use tower_http::trace::{self, TraceLayer};
use tracing::{Level, Span};
use tracing_subscriber::{filter, layer::SubscriberExt, util::SubscriberInitExt};
mod user;
#[derive(Clone)]
struct AppState {
    todo_repository: TodoRepository,
    db_con: PgPool,
    redis_con: MultiplexedConnection,
}

async fn auth(
    State(state): State<AppState>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, HttpError> {
    let token = headers.get("token").unwrap();

    let user_id: i32 = state
        .redis_con
        .clone()
        .get(token.to_str().unwrap().to_string())
        .await
        .unwrap();

    Ok(next.run(request).await)
}
#[tokio::main]
async fn main() {
    let client = redis::Client::open("redis://127.0.0.1/");

    let mut redis_con = client
        .unwrap()
        .get_multiplexed_tokio_connection()
        .await
        .unwrap();

    let filter = filter::Targets::new().with_default(Level::DEBUG);

    let tracing_layer = tracing_subscriber::fmt::layer().with_target(false);

    let _ = tracing_subscriber::registry()
        .with(tracing_layer)
        .with(filter)
        .try_init();

    let logger = TraceLayer::new_for_http()
        .make_span_with(trace::DefaultMakeSpan::default())
        .on_response(|response: &Response<Body>, _: Duration, _: &Span| {
            tracing::info!("{}", response.status())
        });

    let db_con: PgPool = PgPool::connect("postgresql://root:toor@localhost:5432/todo")
        .await
        .unwrap();

    let todo_repository = TodoRepository::new(db_con.clone());

    let state = AppState {
        todo_repository,
        db_con,
        redis_con,
    };

    let auth_routes = AuthRoutes::get(state.clone());

    let app = Router::new().nest("/auth", auth_routes).nest(
        "/todo",
        Router::new()
            .route("/", get(handlers::get_handler))
            .route("/:id", get(handlers::get_one_handler))
            .route("/", post(handlers::create_handler))
            .layer(logger)
            .layer(middleware::from_fn_with_state(state.clone(), auth))
            .with_state(state),
    );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
