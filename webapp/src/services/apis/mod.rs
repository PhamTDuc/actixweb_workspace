pub mod admin;

use std::{collections::HashSet, path::Path};

use actix_files::NamedFile;
use actix_web::{error::{ErrorInternalServerError, ErrorBadRequest, ErrorForbidden}, http::header::{DispositionType, ContentDisposition}};
use actix_session::Session;
use actix_web::{Responder, get, post, web::{self, Data}, HttpRequest, HttpResponse, Error};
use authentication::claims::{Claims, AuthProvider};
use log::{info};
use redis::{Connection, Commands};
use crate::{services::{models::{response::{UserInfoWithPermission, UserInfo, Response, LoginResponse}, request::{UserRegister, User, GetAccessTokenRequest}}}, config::AppData};
use actix_web_grants::proc_macro::{ has_permissions};
use crate::services::models::response::{Role,Status, Permission};
use base64::prelude::{Engine as _, BASE64_URL_SAFE_NO_PAD};

fn get_app_data(req:&HttpRequest)->Result<&Data<AppData>, Error>{
    return req.app_data::<Data<AppData>>().ok_or(ErrorInternalServerError("Failed to get app data"));
}

fn get_redis_conn(app_data:&Data<AppData>) -> Result<Connection, Error>{
    return app_data.redis_client.get_connection().map_err(|_|ErrorInternalServerError("Failed to connect to redis"));
}

fn generate_login_response(user_name:&str, permission: &[Permission], auth_provider: &AuthProvider)->Result<LoginResponse, Error>{
    let claims = Claims::new(user_name.to_string(), Some(permission.into_iter().map(|permission| permission.to_owned().into()).collect()), auth_provider.access_token_expiration); 
    let access_token =  auth_provider.create_jwt(&claims).map_err(|e| ErrorInternalServerError(e))?;
    let refresh_token_claims = Claims::new_token(user_name.to_string(), auth_provider.refresh_token_expiration);
    let refresh_token = auth_provider.create_jwt(&refresh_token_claims).map_err(|e|ErrorInternalServerError(e))?;

    return Ok(LoginResponse{access_token, refresh_token});
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
pub async fn login(req: HttpRequest, info: web::Json<User>)->Result<impl Responder, Error>{
    let app_data = get_app_data(&req)?;
    let user = info.into_inner();
    
    let query = sqlx::query_as!(UserInfoWithPermission, r#"
    SELECT user_info.user_name, user_info.email, user_info.password, 
    permission_role.role AS "role: Role", permission_role.permission AS "permission: Vec<Permission>"
    FROM authentication.user_info INNER JOIN authentication.permission_role 
    ON user_info.role = permission_role.role WHERE user_info.user_name=$1"#, &user.user_name)
    .fetch_one(&app_data.pool).await;

if let Ok(user_info_with_permission)= query{     
    if let Ok(valid) = Claims::verify_password(&app_data.config.secret_key, &user.password, &user_info_with_permission.password){
        if valid {
                    let login_response = generate_login_response(&user.user_name, &user_info_with_permission.permission, &app_data.auth_provider)?;
                    let mut refresh_tokens = HashSet::new();
                    let mut redis_conn = get_redis_conn(&app_data)?;
                    refresh_tokens.insert(login_response.refresh_token.clone());
                    redis_conn.set(user.user_name, &serde_json::to_string(&refresh_tokens).expect("Failed to convert refresh token to json")).map_err(|e|ErrorInternalServerError(e))?;
                    return Ok(HttpResponse::Ok().json(Response::<LoginResponse>::new(true, Some(login_response), None)));
                }
        }
    }
    return Ok(HttpResponse::Ok().json(Response::<LoginResponse>::new(false, None, Some("Login failed".to_string()))).into());
}

#[get("/get_new_access_token")]
pub async fn get_new_access_token(req: HttpRequest, info: web::Json<GetAccessTokenRequest>) -> Result<impl Responder, Error>{
    let app_data = get_app_data(&req)?;
    let refresh_token =  info.into_inner().refresh_token;
    let refresh_token_claims = app_data.auth_provider.decode_jwt(&refresh_token)?;
    let mut redis_conn =get_redis_conn(&app_data)?;
    let refresh_tokens_json:String = redis_conn.get(&refresh_token_claims.user_name).map_err(|e|ErrorForbidden(e))?;
    let mut refresh_tokens:HashSet<String> =  serde_json::from_str(&refresh_tokens_json).map_err(|e|ErrorInternalServerError(e))?;
    let mut redis_conn = get_redis_conn(&app_data)?;
    if refresh_tokens.contains(&refresh_token){
        let query = sqlx::query_as!(UserInfoWithPermission, r#"
        SELECT user_info.user_name, user_info.email, user_info.password, 
        permission_role.role AS "role: Role", permission_role.permission AS "permission: Vec<Permission>"
        FROM authentication.user_info INNER JOIN authentication.permission_role 
        ON user_info.role = permission_role.role WHERE user_info.user_name=$1"#, &refresh_token_claims.user_name)
        .fetch_one(&app_data.pool).await.map_err(|e|ErrorInternalServerError(e))?;

        let login_response = generate_login_response(&query.user_name, &query.permission, &app_data.auth_provider)?;
        refresh_tokens.remove(&refresh_token);
        refresh_tokens.insert(login_response.refresh_token.clone());
        redis_conn.set(query.user_name, &serde_json::to_string(&refresh_tokens).expect("Failed to convert refresh token to json")).map_err(|e|ErrorInternalServerError(e))?;
        return Ok(HttpResponse::Ok().json(Response::<LoginResponse>::new(true, Some(login_response), None)));
    }else{
        redis_conn.del(refresh_token_claims.user_name).map_err(|e|ErrorForbidden(e))?;
        return Ok(HttpResponse::Forbidden().into());
    }
}


#[post("/register")]
pub async fn register(req: HttpRequest, info: web::Json<UserRegister>)->Result<impl Responder, Error>{
    let app_data = get_app_data(&req)?;
    let user_register = info.into_inner();
    let hashed_password =  Claims::hashing_pasword(&app_data.config.secret_key, &user_register.password).map_err(|e| ErrorInternalServerError(e))?;
    let query =  sqlx::query_as!(UserInfo, r#"
        INSERT INTO authentication.user_info (user_name, email, password, role, status)
        VALUES ($1, $2, $3, 'user', 'deactivate') 
        RETURNING id, user_name, email, password, role AS "role: Role", status AS "status: Status""#, 
    &user_register.user_name, &user_register.email, &hashed_password)
    .fetch_one(&app_data.pool).await;

    if let Ok(user_info) = query{
        let mut token =  app_data.auth_provider.create_jwt(&Claims::new_token(user_register.user_name.clone(), app_data.config.otp_token_expiration)).map_err(|e|ErrorInternalServerError(e))?;
        let uuid = BASE64_URL_SAFE_NO_PAD.encode(user_info.id.to_string());
        token = BASE64_URL_SAFE_NO_PAD.encode(token);
        let activate_url = format!("/activate/{}/{}", uuid, token); // TODO: Send this redirect Link via Email instead of redirect
        info!("Activate Link URL: {}", activate_url);
        return  Ok(HttpResponse::Ok().json(Response::<String>::new(true, Some("Register success, please check your email for confirmation".to_string()), None)));
    }

    return Ok(HttpResponse::Ok().json(Response::<String>::new(false, None, Some("Register failed, user name already exists, please try again".to_string()))));
}

#[get("/activate/{uuid}/{token}")]
pub async fn activate(req: HttpRequest, path: web::Path<(String, String)>)->Result<impl Responder, Error>{
    let app_data = get_app_data(&req)?;
    let (uuid, token) = path.into_inner();
    let jwt = String::from_utf8(BASE64_URL_SAFE_NO_PAD.decode(token).map_err(|e| ErrorBadRequest(e))?).map_err(|e| ErrorInternalServerError(e))?;
    let claims = app_data.auth_provider.decode_jwt(&jwt);
    if let Ok(..) = claims {
        let id = String::from_utf8(BASE64_URL_SAFE_NO_PAD.decode(uuid).map_err(|e| ErrorBadRequest(e))?).map_err(|e| ErrorInternalServerError(e))?;
        let query = sqlx::query!(
            r#"UPDATE authentication.user_info
            SET status='active'
            WHERE id=$1 AND status='deactivate'"#, id.parse::<i64>().map_err(|e| ErrorBadRequest(e))?)
            .execute(&app_data.pool).await;
        if query.is_ok() {
            return Ok(HttpResponse::Ok().json(Response::<String>::new(true, Some("Validate New Register success, please login to continue".to_string()), None)));
        }
    }

    return Ok(HttpResponse::ExpectationFailed().json(Response::<String>::new(false, None, Some("Failed to validate new registered user".to_string()))))
}

// #[post("/get_google_access_token")]
// pub async fn get_google_access_token()->impl Responder{
//     let client =  Client::new();
//     let jwt_token = google_services::Claims::create_jwt().expect("Failed to create jwt");
//     let params = [
//         ("grant_type", "urn:ietf:params:oauth:grant-type:jwt-bearer"),
//         ("assertion", &jwt_token)];
//     let res = client.post("https://oauth2.googleapis.com/token")
//         // .content_type("application/x-www-form-urlencoded")
//         .form(&params)
//         .send().await;
    
//     if let Ok(result)=res{
//         let body = result.text().await.expect("Failed to get response");
//         return HttpResponse::Ok().body(body);
//     }
//     else{
//         info!("{:#?}", res);
//     }

//     return HttpResponse::InternalServerError().into();
// }

#[get("/static/{filename:.*}")]
async fn get_file(req:HttpRequest)->Result<NamedFile, Error>{
    let path: std::path::PathBuf = req.match_info().query("filename").parse().unwrap();
    let file = NamedFile::open(Path::new(".").join("static").join(path))?;
    Ok(file
        .use_last_modified(true)
        .set_content_disposition(ContentDisposition {
            disposition: DispositionType::Attachment,
            parameters: vec![],
        }))
}