use serde::Deserialize;

#[derive(Deserialize)]
pub struct UserRegister{
    pub user_name: String,
    pub email: String,
    pub password: String,
}