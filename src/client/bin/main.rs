use rp::client::{RemoteRespoClientFactory, create_client_name};
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
    url: Option<String>,
    #[arg(short, long)]
    name: Option<String>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Locks a pool for maintenance
    Lock,
    /// Locks a pool while the shell command specified as arguments is running
    While { shell_command: Vec<String> },
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
    let server_url = args.url.or_else(|| std::env::var("RP_SERVER").ok());

    match args.command {
        Commands::Lock => {
            let mut factory =
                RemoteRespoClientFactory::new(server_url.expect("No server specified"));
            let mut client = factory.create("test_client".into());

            let ok_request = ResourceRequest {
                by_name: Some(args.name.unwrap()),
                ..Default::default()
            };
            assert!(client.request(&ok_request).await.is_ok());
        }
        Commands::While { shell_command } => {
            let mut factory =
                RemoteRespoClientFactory::new(server_url.expect("No server specified"));
            let mut client = factory.create(create_client_name());

            let ok_request = ResourceRequest {
                by_name: Some(args.name.expect("No pool name specified")),
                ..Default::default()
            };
            let lease = client.request(&ok_request).await;
            match lease {
                Ok(_lease) => {
                    // FIXME: will not using it here cause a drop before we go out of scope?
                    whilerun(shell_command).await.unwrap()
                }
                Err(x) => {
                    println!("An error occured: {:?}", x);
                }
            }
        }
    }
    Ok(())
}
