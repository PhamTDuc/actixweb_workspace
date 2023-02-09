use serde::Serialize;

#[derive(Serialize)]
pub struct Status{
    pub status: String
}

struct UserInfo{
    user_name:  String,
    email: String,
    password: String,
    role: String,
    status: String
}