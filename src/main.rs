// mod agent;
// mod language;
mod agent;
mod utils;

use clap::Parser;
use agent::Agent;

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    ask: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let mut agent = Agent::new();
    let response = agent.run(&args.ask);
    println!("{}", response);
}
