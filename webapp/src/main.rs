use actix_web::{HttpServer, App, web::{Data}};
use log::info;
use sqlx::postgres::PgPoolOptions;


#[actix_web::main]
async fn main()->std::io::Result<()>{
    env_logger::init();
    dotenvy::dotenv().ok();

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
        .app_data(app_data.clone())
        .configure(webapp::config::app_config)
    })
    .workers(config.n_workers)
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}