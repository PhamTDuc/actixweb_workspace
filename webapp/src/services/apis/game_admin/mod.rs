mod request;
mod response;

use actix_web::{Responder, Error, HttpRequest, post, web, HttpResponse, error::{ErrorInternalServerError, ErrorBadRequest}};
use actix_web_grants::proc_macro::has_permissions;
use actix_web_httpauth::extractors::bearer::BearerAuth;
use sqlx::query_as;
use chrono::{Utc,DateTime, NaiveDateTime};

use crate::services::{apis::get_app_data, models::response::{Response}};

// TODO: Find solution to fix auto increment though add_platform or add_game failed  (Expected Behaivour)

#[post("/add_platform")]
#[has_permissions["add_platform"]]
pub async fn add_platform(req: HttpRequest, info: web::Json<request::AddPlatformRequest>)->Result<impl Responder, Error>{
    let app_data = get_app_data(&req)?;
    let request = info.into_inner();
    let query = query_as!(response::PlatformResponse, r#"
    INSERT INTO game_admin.platform (platform)
    VALUES ($1) RETURNING id, platform"#,
    request.platform).fetch_one(&app_data.pool).await.map_err(|e|ErrorBadRequest(e))?;
    return Ok(HttpResponse::Ok().json(Response::new(true, Some(query), None)));
}

#[post("/add_game")]
#[has_permissions["add_game"]]
pub async fn add_game(req: HttpRequest, info: web::Json<request::AddGameRequest>, auth: BearerAuth)->Result<impl Responder, Error>{
    let app_data = get_app_data(&req)?;
    let request = info.into_inner();
    let claim = app_data.auth_provider.decode_jwt(auth.token())?;
    let query = query_as!(response::GameResponse, r#"
        INSERT INTO game_admin.game (name, owner, created_date)
        VALUES ($2, (SELECT id FROM authentication.user_info WHERE $1=user_name), $3) 
        RETURNING id, name, owner, created_date AS "created_date: DateTime<Utc>"
    "#, claim.user_name, request.name, Utc::now().naive_utc())
    .fetch_one(&app_data.pool).await.map_err(|e|ErrorBadRequest(e))?;

    return Ok(HttpResponse::Ok().json(Response::new(true, Some(query), None)));
}