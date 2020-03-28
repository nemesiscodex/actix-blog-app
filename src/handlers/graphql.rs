use deadpool_postgres::Pool;
use juniper::{FieldError, RootNode};
use deadpool_postgres::Client;
use tokio_pg_mapper::FromTokioPostgresRow;
use std::sync::Arc;
use crate::models::user::{User, CreateUser};
use crate::errors::{AppError, AppErrorType};
use crate::config::HashingService;
use tokio_postgres::error::{Error, SqlState};
use slog_scope::error;

#[derive(Clone)]
pub struct Context {
    pub pool: Arc<Pool>,
    pub hashing: Arc<HashingService>
}

/// Context Marker
impl juniper::Context for Context {}


pub struct Query {}

#[juniper::graphql_object(
    Context = Context,
)]
impl Query {
    pub async fn api_version() -> &str {
        "1.0"
    }

    pub async fn users(context: &Context) -> Result<Vec<User>, FieldError> {
        let client: Client = context.pool
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
}

pub struct Mutation {}

#[juniper::graphql_object(
    Context = Context,
)]
impl Mutation {
    async fn create_user(input: CreateUser, context: &Context) -> Result<User, AppError> {

        let client: Client = context.pool
            .get()
            .await
            .map_err(|err| {
                error!("Error getting client {}", err; "query" => "users");
                err
            })?;

        let statement = client
            .prepare("insert into users (username, email, password, bio, image) values ($1, $2, $3, $4, $5) returning *")
            .await?;

        let password_hash = context.hashing.hash(input.password).await?;

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

pub type Schema = RootNode<'static, Query, Mutation>;

pub fn create_schema() -> Schema {
    Schema::new(Query {}, Mutation {})
}
