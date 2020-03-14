use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tokio_pg_mapper_derive::PostgresMapper;

#[derive(Clone, Serialize, Deserialize, PostgresMapper)]
#[pg_mapper(table = "comments")]
pub struct Comment {
    pub id: Uuid,
    pub author_id: Uuid,
    pub post_id: Uuid,
    pub body: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
