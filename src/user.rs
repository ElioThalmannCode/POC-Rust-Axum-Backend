use std::{error::Error, i32};

use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use serde_email::Email;
use serde_json::json;
use sqlx::{error::DatabaseError, query, query_as, PgConnection, Pool, Postgres};

use crate::lib::http_error::HttpError;

#[derive(Deserialize)]
pub struct NewUser {
    email: Email,
    password: String,
}

#[derive(Deserialize, Serialize)]
struct User {
    id: i32,
    email: String,
}

pub enum UserCreationError {
    UserAllreadyExists,
    DatabaseError,
    HashingError,
}

impl From<argon2::password_hash::Error> for UserCreationError {
    fn from(value: argon2::password_hash::Error) -> Self {
        Self::HashingError
    }
}

impl From<sqlx::Error> for UserCreationError {
    fn from(value: sqlx::Error) -> Self {
        Self::DatabaseError
    }
}

impl From<UserCreationError> for HttpError {
    fn from(value: UserCreationError) -> Self {
        match value {
            UserCreationError::UserAllreadyExists => {
                HttpError::conflict("User with this email allready exists.".to_string())
            }
            _ => HttpError::internal_server_error(),
        }
    }
}

pub async fn create_user(user: NewUser, db_con: &Pool<Postgres>) -> Result<i32, UserCreationError> {
    // check if user allready exists
    match get_user_by_email(user.email.clone(), db_con).await {
        Err(UserAquiringError::UserDoesNotExist) => (),
        Ok(_) => return Err(UserCreationError::UserAllreadyExists),
        _ => return Err(UserCreationError::DatabaseError),
    };

    // create password hash
    let salt = SaltString::generate(&mut OsRng);
    let hashed_passwort = Argon2::default()
        .hash_password(user.password.as_bytes(), &salt)?
        .to_string();

    // insert new user into database and return the id
    let id_user: i32 = query!(
        "INSERT INTO users (email, password) VALUES ($1, $2) RETURNING id;",
        user.email.to_string(),
        hashed_passwort,
    )
    .fetch_one(db_con)
    .await?
    .id;

    return Ok(id_user);
}

pub enum UserAquiringError {
    UserDoesNotExist,
    DatabaseError,
}

impl From<sqlx::Error> for UserAquiringError {
    fn from(value: sqlx::Error) -> Self {
        match value {
            sqlx::Error::RowNotFound => UserAquiringError::UserDoesNotExist,
            _ => UserAquiringError::DatabaseError,
        }
    }
}

pub async fn get_user_by_email(
    email: Email,
    db_con: &Pool<Postgres>,
) -> Result<User, UserAquiringError> {
    let user = query_as!(
        User,
        "SELECT id, email FROM users WHERE email = $1 LIMIT 1",
        email.to_string()
    )
    .fetch_one(db_con)
    .await?;

    Ok(user)
}
