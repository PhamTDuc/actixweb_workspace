use serde::Serialize;

#[derive(Serialize)]
pub struct AddPlatformResponse{
    pub id:i32,
    pub platform: String
}