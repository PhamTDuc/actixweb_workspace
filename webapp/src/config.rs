use core::fmt;
use std::{net::{ToSocketAddrs, SocketAddr, IpAddr, Ipv4Addr}, str::FromStr, vec};
use actix_web::web;
use serde::Deserialize;
use config::ConfigError;
use sqlx::{Postgres, Pool};

use crate::services;

#[derive(Deserialize, Debug)]
pub struct Config{
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

// * Each ServerConfig can have it own data, route and services

pub fn app_config(cfg: &mut web::ServiceConfig){
    cfg
    .service(services::get_ready)
    .service(
        web::scope("/admin")
        .service(services::get_ready)
        .configure(admin_config)
    );
}


pub fn admin_config(cfg: &mut web::ServiceConfig){
    cfg.service(services::get_ready);
}