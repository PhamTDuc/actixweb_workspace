use core::fmt;
use actix_web::web;
use serde::Deserialize;
use config::ConfigError;
use sqlx::{Postgres, Pool};

use crate::services;

#[derive(Deserialize, Debug)]
pub(crate) struct Config{
    pub server: ServerConfig,
    pub database_url:String,
    pub max_db_connection:u32,
    pub n_workers:usize,
    pub jwt_secret:String,
    pub secret_key:String,
}


#[derive(Deserialize, Debug)]
pub struct ServerConfig{
    pub host:String,
    pub port: u32
}

impl  fmt::Display for ServerConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f,"ServerConfig({}:{})", self.host, self.port)
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

pub fn app_config(cfg: &mut web::ServiceConfig){
    cfg
    .service(services::get_ready)
    .service(
        web::scope("/admin")
        .service(services::get_ready)
    );
}
