use crate::{cli::Args, crawler::Crawler};
use clap::Parser;
use crawler::{FlagExtractor, LinkExtractor};
use log::{error, info};
use regex::Regex;
use reqwest::blocking::ClientBuilder;
use select::node::Data;
use std::{env, thread, time::Duration};
use url::Url;

mod cli;
mod crawler;
mod test;

fn main() -> eyre::Result<()> {
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    let args = Args::parse();
    let url = Url::parse(&args.url)?;
    let client = ClientBuilder::new().build()?;
    let link_extractor = LinkExtractor::new(client.clone());
    let flag_extractor = FlagExtractor::new(client.clone(), Regex::new(r"MBSD\{[0-9a-zA-Z]+\}")?);
    let crawler = Crawler::new(&link_extractor, url);

    for url in crawler.take(args.pages) {
        if url.as_str().contains(".xml") {
            continue;
        }

        let nodes = match flag_extractor.get_nodes(url.clone()) {
            Ok(nodes) => nodes,
            Err(err) => {
                error!("{}: {:?}", err, err);
                continue;
            }
        };

        for (i, node) in nodes.iter().enumerate() {
            let text = match &node.data {
                Data::Text(text) => text.to_string(),
                _ => unreachable!(),
            };

            info!("Found flag at {}({}): {}", url, i, text);
        }

        thread::sleep(Duration::from_millis(10));
    }

    Ok(())
}
