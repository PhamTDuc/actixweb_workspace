pub mod admin;

use actix_session::Session;
use actix_web::{Responder, get, post, web::{self, Data}, Error, error, http::StatusCode, HttpRequest, HttpResponse};
use authentication::claims::Claims;
use serde::Deserialize;
use crate::{services::models, config::AppData};
use actix_web_grants::proc_macro::{ has_permissions, has_any_permission};

#[derive(Deserialize)]
pub struct UserPermissions {
    pub username: String,
    pub permissions: Vec<String>,
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
pub async fn login(req: HttpRequest, info: web::Json<UserPermissions>)->impl Responder{
    let app_data = req.app_data::<Data<AppData>>().unwrap();
    
    let query = sqlx::query("SELECT role FROM authentication.user_info WHERE user_name=$1")
    .bind(&info.username)
    .fetch_one(&app_data.pool).await;

    if let Ok(_)= query{      
        let user_info = info.into_inner();
        let claims = Claims::new(user_info.username, user_info.permissions, 1);
       
        let jwt_secret = dotenvy::var("JWT_SECRET");
        if let Ok(jwt_secret) = &jwt_secret{
            let jwt =  Claims::create_jwt(jwt_secret, claims);
            if let Ok(jwt) = &jwt{
                return HttpResponse::Ok().body(jwt.to_owned());
            }
        }
    }   
    return HttpResponse::InternalServerError().into();
}