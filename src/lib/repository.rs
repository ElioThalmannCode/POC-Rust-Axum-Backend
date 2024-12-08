use super::http_error::HttpError;

pub trait CrudRepository<Entity, NewEntity> {
    async fn get_all(&self) -> Result<Vec<Entity>, GetAllError>;
    async fn find_one(&self, id: i32) -> Result<Entity, FindOneError>;
    async fn create(&self, item: NewEntity) -> Result<Entity, CreateError>;
}

pub enum GetAllError {
    Unknown,
}

impl Into<HttpError> for GetAllError {
    fn into(self) -> HttpError {
        HttpError::internal_server_error()
    }
}

impl From<sqlx::Error> for GetAllError {
    fn from(_value: sqlx::Error) -> Self {
        GetAllError::Unknown
    }
}

pub enum FindOneError {
    NotInDatabase,
    Unknown,
}

impl Into<HttpError> for FindOneError {
    fn into(self) -> HttpError {
        match self {
            Self::NotInDatabase => HttpError::not_found_error("item does not exist".to_string()),
            _ => HttpError::internal_server_error(),
        }
    }
}
impl From<sqlx::Error> for FindOneError {
    fn from(value: sqlx::Error) -> Self {
        match value {
            sqlx::Error::RowNotFound => FindOneError::NotInDatabase,
            _ => FindOneError::Unknown,
        }
    }
}

pub enum CreateError {
    Unknown,
}

impl From<sqlx::Error> for CreateError {
    fn from(_value: sqlx::Error) -> Self {
        CreateError::Unknown
    }
}
impl Into<HttpError> for CreateError {
    fn into(self) -> HttpError {
        HttpError::internal_server_error()
    }
}
