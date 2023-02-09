use std::io::Error;
use std::io::ErrorKind;

use clap::Parser;
use log::info;
use sqlx::PgConnection;
use sqlx::Connection;
use sqlx::error;
use authentication::claims::Claims;


#[derive(Debug, Parser)]
#[command(author, version, about, long_about=None)]
struct Args{
    command: String,
    args: Option<Vec<String>>
}


async fn create_super_user(user_name: &str, password: &str)->Result<bool, error::Error> {
    let db_url = dotenvy::var("DATABASE_URL").unwrap();
    let mut conn = PgConnection::connect(&db_url).await?;

    let hashed_password = Claims::hashing_pasword(&dotenvy::var("SECRET_KEY").unwrap(), password).unwrap();

    sqlx::query("INSERT INTO authentication.user_info(user_name, password, role, status) VALUES ($1, $2, 'admin', 'active')")
    .bind(user_name)
    .bind(&hashed_password)
    .execute(&mut conn).await.map_err(|e| {println!("{}", e.to_string());e})?;
    info!("Create super user successfully");
    return Ok(true);
}

#[actix_web::main]
async fn main()->std::io::Result<()>{
    let args = Args::parse();
    dotenvy::dotenv().ok();
    env_logger::init();
    
    let config =  webapp::config::Config::from_env().unwrap();
    info!("Command: {:?}", args);
    info!("Config: {:?}", config);

    match &args.command[..]{
        "create_super"=> {
            assert!(args.args.is_some() && args.args.as_ref().unwrap().len()==2, "Arguments missing or invalid");
            let args = args.args.as_ref().unwrap();
            let _ = create_super_user(&args[0], &args[1]).await.map_err(|_| Error::new(ErrorKind::Interrupted, "Failed to create super user"))?;
            return Ok(());
        }
        _=> info!("Invalid command"),
    }
    Ok(())
}