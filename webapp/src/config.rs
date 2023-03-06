use core::fmt;
use std::{net::{ToSocketAddrs, SocketAddr, IpAddr, Ipv4Addr}, str::FromStr, vec};
use actix_web::{web::{self, Data},error::{ErrorUnauthorized}, dev::ServiceRequest, Error};
use actix_web_grants::permissions::AttachPermissions;
use authentication::claims::{Claims, AuthProvider};
use redis::Client;
use serde::Deserialize;
use config::ConfigError;
use sqlx::{Postgres, Pool};
use actix_web_httpauth::{extractors::bearer::BearerAuth, middleware::HttpAuthentication};


use crate::services;

#[derive(Deserialize, Debug, Clone)]
pub struct Config{
    pub server: ServerConfig,
    pub database_url:String,
    pub redis_url:String,
    pub max_db_connection:u32,
    pub n_workers:usize,
    pub jwt_secret:String,
    pub otp_token_expiration: i64,
    pub access_token_expiration: i64,
    pub refresh_token_expiration: i64,
    pub secret_key:String,
    pub cookies_key:String,
}


#[derive(Deserialize, Debug, Clone)]
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
    pub pool: Pool<Postgres>,
    pub redis_client: Client,
    pub config: Config,
    pub auth_provider:AuthProvider
}

pub async fn validator(req: ServiceRequest, cred: BearerAuth)-> Result<ServiceRequest, (Error, ServiceRequest)> {
    let auth_provider = &req.request().app_data::<Data<AppData>>().unwrap().auth_provider;
    let result = jsonwebtoken::decode::<Claims>(&cred.token(), &auth_provider.decoding_key, &auth_provider.validation)
    .map(|data| data.claims)
    .map_err(|e| ErrorUnauthorized(e.to_string()));

    match result {
            Ok(claims) => {
                if let Some(permission) = claims.permissions{
                    req.attach(permission);
                }
                Ok(req)
            }
            // required by `actix-web-httpauth` validator signature
            Err(e) => Err((e, req))
        }

}


// * Each ServerConfig can have it own data, route and services

pub fn app_config(cfg: &mut web::ServiceConfig){
    let auth =  HttpAuthentication::bearer(validator);
    cfg
    .service(services::apis::get_ready)
    .service(services::apis::register)
    .service(services::apis::activate)
    .service(services::apis::login)
    .service(services::apis::get_new_access_token)
    // .service(services::apis::get_google_access_token)
    .service(services::apis::get_file)
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
    cfg.service(services::apis::game_admin::add_platform);
}

