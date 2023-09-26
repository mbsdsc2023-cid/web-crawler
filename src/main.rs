use crate::cli::Args;
use clap::Parser;
use crawler::LinkExtractor;
use log::{error, info};
use reqwest::blocking::ClientBuilder;
use std::env;
use url::Url;

mod cli;
mod crawler;

fn main() -> eyre::Result<()> {
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    let args = Args::parse();
    let url = Url::parse(&args.url)?;
    let client = ClientBuilder::new().build()?;
    let extractor = LinkExtractor::new(client);
    let links = match extractor.get_links(url) {
        Ok(links) => links,
        Err(err) => {
            error!("{}: {:?}", err, err);
            return Ok(());
        }
    };

    for link in links.iter() {
        println!("{}", link);
    }

    info!("OK!");

    Ok(())
}
