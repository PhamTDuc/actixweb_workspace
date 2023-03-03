use serde::Deserialize;

#[derive(Deserialize)]
pub struct AddPlatformRequest{
    pub platform: String
}