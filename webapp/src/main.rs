use actix_web::{HttpServer, App, web::{Data}, cookie::Key};
use actix_session::{SessionMiddleware, storage::CookieSessionStore};
use log::info;
use sqlx::postgres::PgPoolOptions;


#[actix_web::main]
async fn main()->std::io::Result<()>{
    dotenvy::dotenv().ok();
    env_logger::init();

    let config = webapp::config::Config::from_env().unwrap();
    info!("Config: {:?}", config);

    let pool = PgPoolOptions::new()
        .max_connections(config.max_db_connection)
        .connect(&config.database_url)
        .await
        .expect("Database connection failed");

    let app_data = Data::new({
        webapp::config::AppData{
            pool
        }
    });
    

    info!("Starting server at {:}", config.server);
    HttpServer::new(move||{
        App::new()
        .wrap(
            SessionMiddleware::builder(CookieSessionStore::default(), Key::from(&config.cookies_key.as_bytes()))
            // .cookie_domain(Some("webapp".to_owned()))
            // .cookie_content_security(actix_session::config::CookieContentSecurity::Signed)
            .cookie_same_site(actix_web::cookie::SameSite::Lax)
            .build()
        )
        .app_data(app_data.clone())
        .configure(webapp::config::app_config)
    })
    .workers(config.n_workers)
    .bind(config.server)?
    .run()
    .await
}