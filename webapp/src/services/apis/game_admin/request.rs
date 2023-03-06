use serde::Deserialize;

#[derive(Deserialize)]
pub struct AddPlatformRequest{
    pub platform: String
}

#[derive(Deserialize)]
pub struct AddGameRequest{
    pub name: String,                                                                                                                                                                                                                                                                                                                                     
}