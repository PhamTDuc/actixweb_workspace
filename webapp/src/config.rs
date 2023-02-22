use core::fmt;
use std::{net::{ToSocketAddrs, SocketAddr, IpAddr, Ipv4Addr}, str::FromStr, vec};
use actix_web::{web,error, dev::ServiceRequest, Error, http::StatusCode};
use authentication::claims::Claims;
use serde::Deserialize;
use config::ConfigError;
use sqlx::{Postgres, Pool};
use actix_web_httpauth::{extractors::bearer::BearerAuth, middleware::HttpAuthentication};


use crate::services;

#[derive(Deserialize, Debug)]
pub struct Config{
    pub server: ServerConfig,
    pub database_url:String,
    pub max_db_connection:u32,
    pub n_workers:usize,
    pub jwt_secret:String,
    pub secret_key:String,
    pub cookies_key:String,
}


#[derive(Deserialize, Debug)]
pub struct ServerConfig{
    pub host:String,
    pub port: u16
}

impl  fmt::Display for ServerConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"ServerConfig({}:{})", self.host, self.port)
    }
}

impl From<&ServerConfig> for SocketAddr{
    fn from(value: &ServerConfig) -> Self {
        return SocketAddr::new(IpAddr::V4(Ipv4Addr::from_str(&value.host).unwrap()), value.port)
    }
}

impl ToSocketAddrs for ServerConfig{
    type Iter = vec::IntoIter<SocketAddr>;

    fn to_socket_addrs(&self) -> std::io::Result<Self::Iter> {
        let socket_addr:SocketAddr = self.into();
        Ok(vec![socket_addr].into_iter())
    }
}

impl Config {
    pub fn from_env()->Result<Config, ConfigError>{
        config::Config::builder()
        .add_source(config::Environment::default())
        .build()?
        .try_deserialize()
    }
}

pub struct AppData{
    pub pool: Pool<Postgres>
}

pub async fn validator(req: ServiceRequest, cred: BearerAuth)-> Result<ServiceRequest, (Error, ServiceRequest)> {
    let jwt_secret = dotenvy::var("JWT_SECRET");
    match jwt_secret{
        Ok(jwt_secret) => return Claims::validator(&jwt_secret, req, cred).await,
        Err(e) => return Err((error::InternalError::new(e, StatusCode::INTERNAL_SERVER_ERROR).into(), req)),
    }
}


// * Each ServerConfig can have it own data, route and services

pub fn app_config(cfg: &mut web::ServiceConfig){
    let auth =  HttpAuthentication::bearer(validator);
    cfg
    .service(services::apis::get_ready)
    .service(services::apis::login)
    .service(services::apis::get_google_access_token)
    .service(
        web::scope("/api")
        .wrap(auth)
        .service(
            web::scope("/admin")
            .configure(admin_config)
        )
    );
}


pub fn admin_config(cfg: &mut web::ServiceConfig){
    cfg.service(services::apis::get_ready_role);
}

