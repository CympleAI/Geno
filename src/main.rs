#[macro_use]
extern crate tracing;

mod agent;
mod utils;
mod config;

use clap::Parser;
use config::AgentConfig;
use agent::Agent;
use tracing::level_filters::LevelFilter;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Command {
    /// ask info
    #[arg(short, long)]
    ask: String,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::DEBUG)
        .init();

    let args = Command::parse();
    let mut agent = Agent::new(AgentConfig::default()).unwrap();
    let response = agent.run(args.ask).await.unwrap();
    println!("{}", response);
}
