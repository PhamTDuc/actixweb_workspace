pub mod admin;
use log::info;
use actix_session::Session;
use actix_web::{Responder, get, post, web::{self, Data}, Error, error, http::StatusCode, HttpRequest, HttpResponse};
use authentication::claims::Claims;
use serde::Deserialize;
use crate::{services::models::{self, response::{PermissionRole}}, config::AppData};
use actix_web_grants::proc_macro::{ has_permissions, has_any_permission};

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
    
    let query = sqlx::query_as::<_, PermissionRole>("
        WITH user_role as (SELECT role FROM authentication.user_info WHERE user_name=$1)
        SELECT * FROM authentication.permission_role
        WHERE role=(SELECT role FROM user_role)")
    .bind(&info.username)
    .fetch_one(&app_data.pool).await;

    if let Ok(role)= query{      
        let user_info = info.into_inner();
        let claims = Claims::new(user_info.username, role.permission.clone().into_iter().map(|permission| permission.into()).collect(), 1);
       
        let jwt_secret = dotenvy::var("JWT_SECRET");
        if let Ok(jwt_secret) = &jwt_secret{
            let jwt =  Claims::create_jwt(jwt_secret, claims);
            if let Ok(jwt) = &jwt{
                return HttpResponse::Ok().body(serde_json::to_string(&role).unwrap());
            }
        }
    }
    return HttpResponse::Forbidden().into();
}