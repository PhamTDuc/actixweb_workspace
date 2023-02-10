use serde::{Serialize};
use sqlx::{Type, postgres::{PgHasArrayType, PgTypeInfo, types::Oid}};

#[derive(Serialize)]
pub struct Status{
    pub status: String
}

#[derive(sqlx::FromRow)]
pub struct UserInfo{
    pub user_name:  String,
    pub email: String,
    pub password: String,
    pub role: String,
    pub status: String
}

#[derive(sqlx::FromRow, Serialize)]
pub struct PermissionRole{
    pub role:Role,
    pub permission: Option<Vec<Permission>>
}
 
#[derive(sqlx::Type, Serialize, Clone)]
#[sqlx(type_name="role")]
#[sqlx(rename_all="lowercase")]
pub enum Role{
    Admin,
    User
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