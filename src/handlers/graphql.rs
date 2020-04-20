use deadpool_postgres::Pool;
use juniper::RootNode;
use crate::errors::AppError;
use crate::repositories::{post::{PostRepository, PostLoader}, user::UserRepository};
use crate::config::HashingService;
use crate::models::{post::{CreatePost, Post}, user::{User, CreateUser}};
use std::sync::Arc;
use uuid::Uuid;
use chrono::NaiveDateTime;

#[derive(Clone)]
pub struct Context {
    pub pool: Arc<Pool>,
    pub hashing: Arc<HashingService>,
    pub post_loader: PostLoader
}

impl Context {
    pub fn user_repository(&self) -> UserRepository {
        UserRepository::new(self.pool.clone())
    }

    pub fn post_repository(&self) -> PostRepository {
        PostRepository::new(self.pool.clone())
    }
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

    pub async fn users(context: &Context) -> Result<Vec<User>, AppError> {
        context.user_repository().all().await
    }

    pub async fn user(id: Uuid, context: &Context) -> Result<User, AppError> {
        context.user_repository().get(id).await
    }

    pub async fn posts(context: &Context) -> Result<Vec<Post>, AppError> {
        context.post_repository().all().await
    }

    pub async fn post(id: Uuid, context: &Context) -> Result<Post, AppError> {
        context.post_repository().get(id).await
    }
}

#[juniper::graphql_object(
    Context = Context
)]
impl User {
    pub async fn posts(&self, context: &Context) -> Result<Vec<Post>, AppError> {
        // context.post_repository().get_for_user(self.id).await
        context.post_loader.load(self.id).await
    }

    pub fn id(&self) -> Uuid {
        self.id
    }

    pub fn username(&self) -> &str {
        self.username.as_str()
    }

    pub fn email(&self) -> &str {
        self.email.as_str()
    }

    pub fn bio(&self) -> Option<&str> {
        self.bio.as_deref()
    }

    pub fn image(&self) -> Option<&str> {
        self.image.as_deref()
    }

    pub fn created_at(&self) -> NaiveDateTime {
        self.created_at
    }

    pub fn updated_at(&self) -> NaiveDateTime {
        self.updated_at
    }

}

pub struct Mutation {}

#[juniper::graphql_object(
    Context = Context,
)]
impl Mutation {
    pub async fn create_user(input: CreateUser, context: &Context) -> Result<User, AppError> {
        context.user_repository().create(input, context.hashing.clone()).await
    }

    pub async fn create_post(input: CreatePost, context: &Context) -> Result<Post, AppError> {
        context.post_repository().create(input).await
    }
}

pub type Schema = RootNode<'static, Query, Mutation>;

pub fn create_schema() -> Schema {
    Schema::new(Query {}, Mutation {})
}
