use deadpool_postgres::{Pool, Client};
use std::sync::Arc;
use slog_scope::error;
use crate::models::post::{Post, CreatePost};
use tokio_pg_mapper::FromTokioPostgresRow;
use crate::errors::{AppError, AppErrorType};
use tokio_postgres::error::{Error, SqlState};
use uuid::Uuid;

pub struct PostRepository {
    pool: Arc<Pool>
}

impl PostRepository {

    pub fn new(pool: Arc<Pool>) -> PostRepository {
        PostRepository { pool }
    }

    pub async fn get(&self, id: Uuid) -> Result<Post, AppError> {
        let client: Client = self.pool
            .get()
            .await
            .map_err(|err| {
                error!("Error getting client {}", err; "query" => "get");
                err
            })?;

        let statement = client.prepare("select * from posts where id = $1").await?;

        client
            .query(&statement, &[&id])
            .await
            .map_err(|err| {
                error!("Error getting users. {}", err; "query" => "get");
                err
            })?
            .iter()
            .map(|row| Post::from_row_ref(row))
            .collect::<Result<Vec<Post>, _>>()?
            .pop()
            .ok_or(AppError {
                cause: None,
                message: None,
                error_type: AppErrorType::NotFoundError
            })
    }

    pub async fn all(&self) -> Result<Vec<Post>, AppError> {
        let client: Client = self.pool
            .get()
            .await
            .map_err(|err| {
                error!("Error getting client {}", err; "query" => "all");
                err
            })?;

        let statement = client.prepare("select * from posts").await?;

        let users = client
            .query(&statement, &[])
            .await
            .map_err(|err| {
                error!("Error getting users. {}", err; "query" => "all");
                err
            })?
            .iter()
            .map(|row| Post::from_row_ref(row))
            .collect::<Result<Vec<Post>, _>>()
            .map_err(|err| {
                error!("Error getting parsing users. {}", err; "query" => "all");
                err
            })?;

        Ok(users)
    }

    pub async fn create(&self, input: CreatePost) -> Result<Post, AppError> {
        let client: Client = self.pool
            .get()
            .await
            .map_err(|err| {
                error!("Error getting client {}", err; "query" => "create");
                err
            })?;

        let statement = client
            .prepare("insert into posts (author_id, slug, title, description, body) values ($1, $2, $3, $4, $5) returning *")
            .await?;

        let slug = match input.slug {
            Some(s) => s,
            None => Uuid::new_v4().to_string()
        };

        let post = client.query(&statement, &[
                &input.author_id,
                &slug,
                &input.title,
                &input.description,
                &input.body
            ])
            .await
            .map_err(|err: Error| {
                match err.code() {
                    Some(code) => match code {
                        c if c == &SqlState::UNIQUE_VIOLATION => AppError {
                            cause: Some(err.to_string()),
                            message: Some(format!("Slug {} already exists.", slug)),
                            error_type: AppErrorType::InvalidField
                        },
                        c if c == &SqlState::FOREIGN_KEY_VIOLATION=> AppError {
                            cause: Some(err.to_string()),
                            message: Some(format!("Author with id {} doesn't exists.", slug)),
                            error_type: AppErrorType::InvalidField
                        },
                        _ => AppError::from(err)
                    }
                    _ => AppError::from(err)
                }
            })?
            .iter()
            .map(|row| Post::from_row_ref(row))
            .collect::<Result<Vec<Post>, _>>()?
            .pop()
            .ok_or(AppError {
                message: Some("Error creating Post.".to_string()),
                cause: Some("Unknown error.".to_string()),
                error_type: AppErrorType::DbError,
            })?;

        Ok(post)
    }
}