use serde::Deserialize;

#[derive(Deserialize)]
pub struct UserRegister{
    pub user_name: String,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct User {
    pub user_name: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct GetAccessTokenRequest{
    pub refresh_token: String,
}