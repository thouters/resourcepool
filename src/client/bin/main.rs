//use rp::respo::{Inventory, Pool, Resource, ResourceRequest};
//use crate::client::{ Resource, RespoClientFactory };

use clap::{Parser, Subcommand};

/// Resource pool client tool
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    #[arg(short, long)]
    server_url: Option<String>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Locks a pool for maintenance
    Lock { pool_name: Option<String> },
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args = Cli::parse();

    match args.command {
        Commands::Lock { pool_name } => {
            let server_url = args.server_url.or_else(|| std::env::var("RP_SERVER").ok());

            server_url.expect("No server specified");

            //let factory = RespoClientFactory::new(server_url);

            println!("lock {:?}", pool_name);
        }
    }
    Ok(())
}
