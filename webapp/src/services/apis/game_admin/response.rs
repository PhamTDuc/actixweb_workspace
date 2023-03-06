use chrono::{Utc, DateTime};
use serde::Serialize;

#[derive(Serialize)]
pub struct PlatformResponse{
    pub id:i32,
    pub platform: String
}

#[derive(Serialize)]
pub struct GameResponse{
    pub id:i64,
    pub name: String,
    pub owner: i64,
    pub created_date: DateTime<Utc>
}