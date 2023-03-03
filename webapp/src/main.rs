use actix_cors::Cors;
use actix_web::{HttpServer, App, web::{Data}, cookie::{Key}};
use actix_session::{SessionMiddleware, storage::CookieSessionStore};
use authentication::claims::AuthProvider;
use log::info;
use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
use sqlx::postgres::PgPoolOptions;
use webapp::config::{AppData, Config};


#[cfg(feature="use_https")]
async fn run_server(config: Config, app_data: Data<AppData>)->std::io::Result<()>{
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder.set_private_key_file("key.pem", SslFiletype::PEM).unwrap();
    builder.set_certificate_chain_file("cert.pem").unwrap();

    return HttpServer::new(move||{

        let cors = Cors::default()
            .allowed_methods(vec!["GET", "POST"])
            // .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .send_wildcard();
            // .allowed_origin(origin);
        
        let session_middleware = 
        SessionMiddleware::builder(CookieSessionStore::default(), Key::from(&config.cookies_key.as_bytes()))
        // .cookie_domain(Some("webapp".to_owned()))
        // .cookie_content_security(actix_session::config::CookieContentSecurity::Signed)
        .cookie_same_site(actix_web::cookie::SameSite::Lax)
        .build();

        App::new()
        .wrap(cors)
        .wrap(session_middleware)
        .app_data(app_data.clone())
        .configure(webapp::config::app_config)
    })
    .workers(config.n_workers)
    .bind_openssl(config.server, builder)?
    .run()
    .await
}

#[cfg(not(feature="use_https"))]
async fn run_server(config: Config, app_data: Data<AppData>)->std::io::Result<()>{

    return HttpServer::new(move||{

        let cors = Cors::default()
            .allowed_methods(vec!["GET", "POST"])
            // .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .send_wildcard();
            // .allowed_origin(origin);
        
        let session_middleware = 
        SessionMiddleware::builder(CookieSessionStore::default(), Key::from(&config.cookies_key.as_bytes()))
        // .cookie_domain(Some("webapp".to_owned()))
        // .cookie_content_security(actix_session::config::CookieContentSecurity::Signed)
        .cookie_same_site(actix_web::cookie::SameSite::Lax)
        .build();

        App::new()
        .wrap(cors)
        .wrap(session_middleware)
        .app_data(app_data.clone())
        .configure(webapp::config::app_config)
    })
    .workers(config.n_workers)
    .bind(config.server)?
    .run()
    .await
}


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

    let redis_client = redis::Client::open(config.redis_url.clone()).unwrap();

    let auth_provider = AuthProvider::new(&config.jwt_secret, config.access_token_expiration, config.refresh_token_expiration);

    let app_data = Data::new({
        AppData{
            pool,
            redis_client: redis_client,
            config: config.clone(),
            auth_provider
        }
    });

    info!("Starting server at {:}", config.server);
    return run_server(config, app_data).await;
   
}