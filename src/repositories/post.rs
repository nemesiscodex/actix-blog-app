use deadpool_postgres::{Pool, Client};
use std::{collections::HashMap, sync::Arc};
use slog_scope::{error, info};
use crate::models::post::{Post, CreatePost};
use tokio_pg_mapper::FromTokioPostgresRow;
use crate::errors::{AppError, AppErrorType};
use tokio_postgres::error::{Error, SqlState};
use uuid::Uuid;
use async_trait::async_trait;
use dataloader::{BatchFn, cached::Loader};

pub struct PostRepository {
    pool: Arc<Pool>,
}

pub struct PostBatcher {
    pool: Arc<Pool>,
}

pub type PostLoader = Loader<Uuid, Vec<Post>, AppError, PostBatcher>;

pub fn get_post_loader(pool: Arc<Pool>) -> PostLoader {
    Loader::new(PostBatcher { pool })
        // https://github.com/cksac/dataloader-rs/issues/12
        .with_yield_count(100)
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

    #[allow(dead_code)]
    pub async fn get_for_user(&self, user_id: Uuid) -> Result<Vec<Post>, AppError> {
        let client: Client = self.pool
            .get()
            .await
            .map_err(|err| {
                error!("Error getting client {}", err; "query" => "get_for_user");
                err
            })?;

        let statement = client.prepare("select * from posts where author_id = $1").await?;

        let users = client
            .query(&statement, &[&user_id])
            .await
            .map_err(|err| {
                error!("Error getting users. {}", err; "query" => "get_for_user");
                err
            })?
            .iter()
            .map(|row| Post::from_row_ref(row))
            .collect::<Result<Vec<Post>, _>>()
            .map_err(|err| {
                error!("Error getting parsing users. {}", err; "query" => "get_for_user");
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

impl PostBatcher {
    pub async fn get_posts_by_user_ids(&self, hashmap: &mut HashMap<Uuid, Vec<Post>>, ids: Vec<Uuid>) -> Result<(), AppError> {
        let client: Client = self.pool
            .get()
            .await
            .map_err(|err| {
                error!("Error getting client {}", err; "query" => "get_posts_by_user_ids");
                err
            })?;

        let statement = client.prepare("select * from posts where author_id = ANY($1)").await?; 

        client
            .query(&statement, &[&ids])
            .await
            .map_err(|err| {
                error!("Error getting posts. {}", err; "query" => "get_posts_by_user_ids");
                err
            })?
            .iter()
            .map(|row| Post::from_row_ref(row))
            .collect::<Result<Vec<Post>, _>>()
            .map_err(|err| {
                error!("Error getting parsing posts. {}", err; "query" => "get_posts_by_user_ids");
                err
            })?
            .iter()
            .fold(
                hashmap,
                |map: &mut HashMap<Uuid, Vec<Post>>, post: &Post| {
                    let vec = map
                        .entry(post.author_id)
                        .or_insert_with(|| Vec::<Post>::new());
                    vec.push(post.clone());
                    map
                }
            );

        Ok(())

    }
}

#[async_trait]
impl BatchFn<Uuid, Vec<Post>> for PostBatcher {
    type Error = AppError;

    async fn load(&self, keys: &[Uuid]) -> HashMap<Uuid, Result<Vec<Post>, AppError>> {

        info!("Loading batch {:?}", keys);

        let mut posts_map = HashMap::new();

        let result: Result<(), AppError> = self.get_posts_by_user_ids(&mut posts_map, keys.into()).await;

        keys
            .iter()
            .map(move |id| {
                let entry = 
                    posts_map.entry(*id)
                        .or_insert_with(|| vec![])
                        .clone();

                    (id.clone(), result.clone().map(|_| entry))
                })
                .collect::<HashMap<_, _>>()
    }
}