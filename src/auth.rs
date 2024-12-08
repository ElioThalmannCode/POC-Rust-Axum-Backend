use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use axum::{extract::State, routing::post, Json, Router};
use rand::distributions::{Alphanumeric, DistString};
use redis::aio::MultiplexedConnection;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::query;

use crate::{
    lib::http_error::HttpError,
    user::{create_user, NewUser},
    AppState,
};

pub struct AuthRoutes {}

impl AuthRoutes {
    pub fn get(state: AppState) -> Router {
        let router: Router = Router::new()
            .route("/login", post(login))
            .route("/register", post(register))
            .with_state(state);

        return router;
    }
}

#[derive(Deserialize, Serialize)]
struct NewRegister {
    email: String,
    password: String,
}

#[derive(Deserialize, Serialize)]
struct User {
    id: i32,
    email: String,
    password: String,
}
#[derive(Deserialize)]
struct LoginInformation {
    email: String,
    password: String,
}
#[axum::debug_handler]
async fn login(
    State(state): State<AppState>,
    Json(login): Json<LoginInformation>,
) -> Result<Json<String>, HttpError> {
    let res = query!(
        "SELECT id, password FROM users WHERE email = ($1)",
        login.email
    )
    .fetch_one(&state.db_con)
    .await
    .unwrap();

    let parsed_hash = PasswordHash::new(&res.password);

    let passwort_check =
        Argon2::default().verify_password(login.password.as_bytes(), &parsed_hash.unwrap());

    match passwort_check {
        Ok(_) => {
            let token = Alphanumeric.sample_string(&mut rand::thread_rng(), 125);
            let _: () = redis::cmd("SET")
                .arg(token.clone())
                .arg(res.id)
                .query_async(&mut state.redis_con.clone())
                .await
                .unwrap();

            return Ok(Json(token));
        }
        Err(_) => {
            return Err(HttpError::unauthorized("lalalal".to_string()));
        }
    }
}

#[derive(Serialize)]
struct RegisterResponse {
    token: String,
}

async fn register(
    // TODO: refactor into service, check if user exists, return token
    State(state): State<AppState>,
    Json(new_user): Json<NewUser>,
) -> Result<Json<RegisterResponse>, HttpError> {
    let user_id = create_user(new_user, &state.db_con).await?;

    let token = create_auth_token(user_id, &mut state.redis_con.clone()).await;

    Ok(Json(RegisterResponse { token }))
}

async fn create_auth_token(user_id: i32, redis_con: &mut MultiplexedConnection) -> String {
    let token = Alphanumeric.sample_string(&mut rand::thread_rng(), 125);

    let _: () = redis::cmd("SET")
        .arg(token.clone())
        .arg(user_id)
        .query_async(redis_con)
        .await
        .unwrap();
    return token;
}
