use clap::Parser;


#[derive(Debug, Parser)]
#[command(author, version, about, long_about=None)]
struct Args{
    command: Option<String>,
}

fn main(){
    let args = Args::parse();
    dotenvy::dotenv().ok();

    let config =  webapp::config::Config::from_env().unwrap();
    println!("Command: {:#?}", args);
    println!("Config: {:#?}", config);
}