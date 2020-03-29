use deadpool_postgres::{Pool, Client};
use std::sync::Arc;
use slog_scope::error;
use crate::models::user::{User, CreateUser};
use tokio_pg_mapper::FromTokioPostgresRow;
use crate::{config::HashingService, errors::{AppError, AppErrorType}};
use tokio_postgres::error::{Error, SqlState};
use uuid::Uuid;

pub struct UserRepository {
    pool: Arc<Pool>
}

impl UserRepository {

    pub fn new(pool: Arc<Pool>) -> UserRepository {
        UserRepository { pool }
    }

    pub async fn get(&self, id: Uuid) -> Result<User, AppError> {
        let client: Client = self.pool
            .get()
            .await
            .map_err(|err| {
                error!("Error getting client {}", err; "query" => "user");
                err
            })?;

        let statement = client.prepare("select * from users where id = $1").await?;

        client
            .query(&statement, &[&id])
            .await
            .map_err(|err| {
                error!("Error getting users. {}", err; "query" => "user");
                err
            })?
            .iter()
            .map(|row| User::from_row_ref(row))
            .collect::<Result<Vec<User>, _>>()?
            .pop()
            .ok_or(AppError {
                cause: None,
                message: None,
                error_type: AppErrorType::NotFoundError
            })
    }

    pub async fn all(&self) -> Result<Vec<User>, AppError> {
        let client: Client = self.pool
            .get()
            .await
            .map_err(|err| {
                error!("Error getting client {}", err; "query" => "users");
                err
            })?;

        let statement = client.prepare("select * from users").await?;

        let users = client
            .query(&statement, &[])
            .await
            .map_err(|err| {
                error!("Error getting users. {}", err; "query" => "users");
                err
            })?
            .iter()
            .map(|row| User::from_row_ref(row))
            .collect::<Result<Vec<User>, _>>()
            .map_err(|err| {
                error!("Error getting parsing users. {}", err; "query" => "users");
                err
            })?;

        Ok(users)
    }

    pub async fn create(&self, input: CreateUser, hashing: Arc<HashingService>) -> Result<User, AppError> {
        let client: Client = self.pool
            .get()
            .await
            .map_err(|err| {
                error!("Error getting client {}", err; "query" => "create");
                err
            })?;

        let statement = client
            .prepare("insert into users (username, email, password, bio, image) values ($1, $2, $3, $4, $5) returning *")
            .await?;

        let password_hash = hashing.hash(input.password).await?;

        let user = client.query(&statement, &[
                &input.username,
                &input.email,
                &password_hash,
                &input.bio,
                &input.image
            ])
            .await
            .map_err(|err: Error| {
                let unique_error = err.code()
                    .map(|code| code == &SqlState::UNIQUE_VIOLATION);

                match unique_error {
                    Some(true) => AppError {
                            cause: Some(err.to_string()),
                            message: Some("Username or email address already exists.".to_string()),
                            error_type: AppErrorType::InvalidField
                        },
                    _ => AppError::from(err)
                }
            })?
            .iter()
            .map(|row| User::from_row_ref(row))
            .collect::<Result<Vec<User>, _>>()?
            .pop()
            .ok_or(AppError {
                message: Some("Error creating User.".to_string()),
                cause: Some("Unknown error.".to_string()),
                error_type: AppErrorType::DbError,
            })?;

        Ok(user)
    }
}