use std::env;

use anyhow::Result;
use clap::Parser;
use fetcher::Fetcher;

mod fetcher;

#[derive(Debug, Parser)]
pub struct Command {
    #[clap(short, long, env)]
    path: Option<String>,

    #[clap(short, long)]
    url: String,
}

impl Command {
    pub async fn execute(self) -> Result<()> {
        let path = self.path.unwrap_or("/tmp/side_chain_data".to_string());
        let mut fetcher = Fetcher::new(path.into(), &self.url)?;
        fetcher.run().await
    }
}
#[tokio::main]
async fn main() -> Result<()> {
    if env::var_os("RUST_BACKTRACE").is_none() {
        env::set_var("RUST_BACKTRACE", "full");
    }
    if env::var_os("RUST_LOG").is_none() {
        env::set_var("RUST_LOG", "info");
    }

    env_logger::init();

    let cmd = Command::parse();

    cmd.execute().await
}
