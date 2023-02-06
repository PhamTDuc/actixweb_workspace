mod models;

use actix_web::{Responder, get};

#[get("/")]
pub async fn get_ready()->impl Responder{
    actix_web::HttpResponse::Ok()
    .json(models::response::Status{
        status:"Ready".to_owned()
    })
}