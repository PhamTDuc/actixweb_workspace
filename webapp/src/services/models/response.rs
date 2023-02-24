use serde::{Serialize};
use sqlx::{Type, postgres::{PgHasArrayType, PgTypeInfo, types::Oid}};

#[derive(sqlx::FromRow)]
pub struct UserInfo{
    pub id: i64,
    pub user_name:  String,
    pub email: Option<String>,
    pub password: String,
    pub role: Role,
    pub status: Status
}

#[derive(sqlx::FromRow)]
pub struct UserInfoWithPermission{
    pub user_name: String,
    pub email: Option<String>,
    pub password: String,
    pub role:Role,
    pub permission: Vec<Permission>
}
 
#[derive(sqlx::Type, Serialize, Clone)]
#[sqlx(type_name="role")]
#[sqlx(rename_all="lowercase")]
pub enum Role{
    Admin,
    User
}

#[derive(sqlx::Type, Serialize, Clone)]
#[sqlx(type_name="status")]
#[sqlx(rename_all="lowercase")]
pub enum Status{
    Active,
    Deactivate,
    Blocked
}


#[derive(sqlx::Type, Serialize, Clone)]
#[sqlx(type_name="permission")]
#[sqlx(rename_all="snake_case")]
pub enum Permission{

    GrantPermission,
    CanView
}

impl From<Permission> for String{
    fn from(value: Permission) -> Self {
        match value {
            Permission::CanView=> "can_view".to_owned(),
            Permission::GrantPermission => "grant_permission".to_owned(),
        }
    }
}

// impl PgHasArrayType for Permission{
//     fn array_type_info() -> sqlx::postgres::PgTypeInfo {
//         PgTypeInfo::with_name("_permission")
//     }
// }

#[derive(Serialize)]
pub struct Response<T> where T:Serialize{
    pub success: bool,
    pub data: Option<T>,
    pub message: Option<String>
}

impl<T:Serialize> Response<T>{
    pub fn new(success: bool, data: Option<T>, message: Option<String>)->Self{
        Self{success, data , message}
    }
}

impl Default for Response<String>{
    fn default() -> Self {
        Self { success: true, data: Some("Success".to_string()), message: None }
    }

}

#[derive(Serialize)]
pub struct LoginResponse{
    pub access_token: String,
    pub refresh_token: String,
}
