pub mod admin;

use actix_session::Session;
use actix_web::{Responder, get, post, web::{self, Data}, HttpRequest, HttpResponse};
use authentication::claims::Claims;
use log::{info, error};
use reqwest::Client;
use serde::Deserialize;
use crate::{services::{models::{response::{PermissionRole, UserInfo, Response}, request::UserRegister}, google_services}, config::AppData};
use actix_web_grants::proc_macro::{ has_permissions};
use crate::services::models::response::{Role,Status, Permission};
use base64::prelude::{Engine as _, BASE64_URL_SAFE_NO_PAD};

#[derive(Deserialize)]
pub struct User {
    pub username: String,
}

#[get("/")]
pub async fn get_ready(session: Session)->impl Responder{
    session.insert("counter", 1000).unwrap();
    return HttpResponse::Ok().json(Response::<String>::default())
}

#[get("/")]
#[has_permissions("Admin")]
pub async fn get_ready_role()->impl Responder{
    return HttpResponse::Ok().json(Response::<String>::default())
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
            let jwt =  app_data.auth_provider.create_jwt(&claims).expect("Failed to create JWT");
            return HttpResponse::Ok().json(Response::<String>::new(true, Some(jwt), None));
        } 
    }
    return HttpResponse::Forbidden().json(Response::<String>::new(false, None, Some("Login failed".to_string()))).into();
}

// TODO: Check user_name and email already exists 
#[post("/register")]
pub async fn register(req: HttpRequest, info: web::Json<UserRegister>)->impl Responder{
    let app_data = req.app_data::<Data<AppData>>().unwrap();
    let user_register = info.into_inner();
    let hashed_password =  Claims::hashing_pasword(&app_data.config.secret_key, &user_register.password).expect("Failed to hashing user password");
    let query =  sqlx::query_as!(UserInfo, r#"
        INSERT INTO authentication.user_info (user_name, email, password, role, status)
        VALUES ($1, $2, $3, 'user', 'deactivate') RETURNING id, user_name, email, password, role AS "role: Role", status AS "status: Status""#, &user_register.user_name, &user_register.email, &hashed_password)
    .fetch_one(&app_data.pool).await.map_err(|e|error!("{}", e.to_string()));


    if let Ok(user_info) = query{
        let mut token =  app_data.auth_provider.create_jwt(&Claims::new(user_register.user_name.clone(), vec![], 60)).expect("Token for email verification");
        let uuid = BASE64_URL_SAFE_NO_PAD.encode(user_info.id.to_string());
        token = BASE64_URL_SAFE_NO_PAD.encode(token);
        let activate_url = format!("/activate/{}/{}", uuid, token); // TODO: Send this redirect Link via Email instead of redirect
        info!("Activate Link URL: {}", activate_url);
        return  HttpResponse::Ok().json(Response::<String>::new(true, Some("Register success, please check your email for confirmation".to_string()), None))
    }

    return HttpResponse::Ok().json(Response::<String>::new(false, None, Some("Register failed, please try again".to_string())));
}

#[get("/activate/{uuid}/{token}")]
pub async fn activate(req: HttpRequest, path: web::Path<(String, String)>)->impl Responder{
    let auth_provider = &req.app_data::<Data<AppData>>().unwrap().auth_provider;
    let (uuid, token) = path.into_inner();
    let app_data = req.app_data::<Data<AppData>>().unwrap();
    let jwt = String::from_utf8(BASE64_URL_SAFE_NO_PAD.decode(token).expect("Failed to decode JWT")).expect("Failed to decode JWT to UTF-8");
    let claims = auth_provider.decode_jwt(&jwt);
    if let Ok(..) = claims {
        let id = String::from_utf8(BASE64_URL_SAFE_NO_PAD.decode(uuid).expect("Failed to decode uuid")).expect("Failed to decode UUID to UTF-8");
        let query = sqlx::query!(
            r#"UPDATE authentication.user_info
            SET status='active'
            WHERE id=$1"#, id.parse::<i64>().expect("Failed to parse id string to id"))
            .execute(&app_data.pool).await;
        if let Ok(..)= query {
            return HttpResponse::Ok().json(Response::<String>::new(true, Some("Validate New Register success, please login to continue".to_string()), None));
        }
    }

    return HttpResponse::ExpectationFailed().json(Response::<String>::new(false, None, Some("Failed to validate new registered user".to_string())))
}

#[post("/get_google_access_token")]
pub async fn get_google_access_token()->impl Responder{
    let client =  Client::new();
    let jwt_token = google_services::Claims::create_jwt().expect("Failed to create jwt");
    let params = [
        ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
        ("assertion", &jwt_token)];
    let res = client.post("https://oauth2.googleapis.com/token")
        // .content_type("application/x-www-form-urlencoded")
        .form(&params)
        .send().await;
    
    if let Ok(result)=res{
        let body = result.text().await.expect("Failed to get response");
        return HttpResponse::Ok().body(body);
    }
    else{
        info!("{:#?}", res);
    }

    return HttpResponse::InternalServerError().into();
}