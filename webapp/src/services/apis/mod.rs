pub mod admin;
use actix_session::Session;
use actix_web::{Responder, get, post, web::{self, Data}, Error, error, http::StatusCode, HttpRequest, HttpResponse};
use authentication::claims::Claims;
use serde::Deserialize;
use crate::{services::models::{self, response::{PermissionRole}}, config::AppData};
use actix_web_grants::proc_macro::{ has_permissions, has_any_permission};
use crate::services::models::response::{Role, Permission};

#[derive(Deserialize)]
pub struct User {
    pub username: String,
}

#[get("/")]
pub async fn get_ready(session: Session)->impl Responder{
    session.insert("counter", 1000).unwrap();
    actix_web::HttpResponse::Ok()
    .json(models::response::Status{
        status:"Ready".to_owned()
    })
}

#[get("/")]
#[has_permissions("Admin")]
pub async fn get_ready_role()->impl Responder{
    actix_web::HttpResponse::Ok()
    .json(models::response::Status{
        status:"Ready".to_owned()
    })
}


#[post("/login")]
pub async fn login(req: HttpRequest, info: web::Json<User>)->impl Responder{
    let app_data = req.app_data::<Data<AppData>>().unwrap();
    
    let query = sqlx::query_as!(PermissionRole, r#"
        WITH user_role AS (SELECT role FROM authentication.user_info WHERE user_name=$1)
        SELECT role AS "role: Role", permission as "permission?: Vec<Permission>" FROM authentication.permission_role
        WHERE role=(SELECT role FROM user_role)"#, &info.username)
    .fetch_one(&app_data.pool).await;

    if let Ok(permission_role)= query{     
        if let Some(permission) = permission_role.permission {
            let user_info = info.into_inner();
            let claims = Claims::new(user_info.username, permission.into_iter().map(|e| e.into()).collect(), 1); 
            let jwt_secret = dotenvy::var("JWT_SECRET");
            if let Ok(jwt_secret) = &jwt_secret{
                let jwt =  Claims::create_jwt(jwt_secret, claims);
                if let Ok(jwt) = &jwt{
                    return HttpResponse::Ok().body(jwt.to_string());
                }
            }
            
        } 
    }
    return HttpResponse::Forbidden().into();
}