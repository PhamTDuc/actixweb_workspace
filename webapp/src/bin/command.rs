use clap::Parser;

#[derive(Debug, Parser)]
#[command(author, version, about, long_about=None)]
struct Args{
    command: Option<String>,

}

fn main(){
    let args = Args::parse();
    println!("Command: {:?}", args);
}