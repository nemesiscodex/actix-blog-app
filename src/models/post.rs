use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tokio_pg_mapper_derive::PostgresMapper;
use juniper::{GraphQLObject, GraphQLInputObject};

#[derive(Clone, Serialize, Deserialize, PostgresMapper, GraphQLObject)]
#[pg_mapper(table = "posts")]
pub struct Post {
    pub id: Uuid,
    pub author_id: Uuid,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub body: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(GraphQLInputObject)]
pub struct CreatePost {
    pub author_id: Uuid,
    pub slug: Option<String>,
    pub title: String,
    pub description: String,
    pub body: String,
}