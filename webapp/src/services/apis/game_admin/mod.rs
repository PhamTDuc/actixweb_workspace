mod request;
mod response;

use actix_web::{Responder, Error, HttpRequest, post, web, HttpResponse, error::ErrorInternalServerError};
use actix_web_grants::proc_macro::has_permissions;
use sqlx::query_as;

use crate::services::{apis::get_app_data, models::response::{Response, LoginResponse}};


#[post("/add_platform")]
#[has_permissions["Add Platform"]]
pub async fn add_platform(req: HttpRequest, info: web::Json<request::AddPlatformRequest>)->Result<impl Responder, Error>{
    let app_data = get_app_data(&req)?;
    let request = info.into_inner();
    let query = query_as!(response::AddPlatformResponse, r#"
    INSERT INTO game_admin.platform (platform)
    VALUES ($1) RETURNING id, platform"#,
    request.platform).fetch_one(&app_data.pool).await.map_err(|e|ErrorInternalServerError(e))?;
    return Ok(HttpResponse::Ok().json(Response::new(true, Some(query), None)));
}