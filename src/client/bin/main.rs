use rp::client::RemoteRespoClientFactory;
use rp::inventory::ClientResourceRequest;
use rp::inventory::ResourceRequest;
use std::{error::Error, process::Command};

use clap::{Parser, Subcommand};

/// Resource pool client tool
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    #[arg(short, long)]
    spec_file: Option<String>,
    #[arg(short, long)]
    server_url: Option<String>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Locks a pool for maintenance
    Lock {
        pool_name: Option<String>,
    },
    While {
        shell_command: Vec<String>,
    },
}

async fn whilerun(shell_command: Vec<String>) -> Result<(), Box<dyn Error>> {
    // Setup command
    let mut command = Command::new(shell_command[0].clone());
    command.args(shell_command[1..].iter());

    // Pipe stdin so we can issue commands to tell Python what we want it to do

    // Spawn the process & take ownership of stdin
    let mut child = command.spawn()?;

    // Make sure the process has exited before we exit
    child.wait()?;

    Ok(())
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let args = Cli::parse();

    match args.command {
        Commands::Lock { pool_name } => {
            let server_url = args
                .server_url
                .or_else(|| std::env::var("RP_SERVER").ok())
                .expect("No server specified");

            let mut factory = RemoteRespoClientFactory::new(server_url);
            let mut client = factory.create("test_client".into());

            let ok_request = ResourceRequest {
                by_name: Some(pool_name.unwrap()),
                ..Default::default()
            };
            assert!(client.request(&ok_request).await.is_ok());
        }
        Commands::While { shell_command } => {
            //            todo!("implement the locking then run  {:?}", shell_command);
            whilerun(shell_command).await.unwrap();
        }
    }
    Ok(())
}
