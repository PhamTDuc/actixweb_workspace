use serde::Deserialize;
use validator::Validate;

#[derive(Deserialize, Validate)]
pub struct UserRegister{
    pub user_name: String,
    #[validate(email)]
    pub email: String,
    #[validate(length(min=8))]
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