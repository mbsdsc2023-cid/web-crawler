use std::env;
use std::sync::Arc;
use std::thread;

use crate::crawler::Crawler;
use crate::crawler::FlagExtractor;
use crate::crawler::LinkExtractor;

use actix_cors::Cors;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use log::{error, info};
use regex::Regex;
use reqwest::blocking::ClientBuilder;
use select::node::Data;
use serde::Deserialize;
use serde::Serialize;
use url::Url;

mod cli;
mod crawler;
mod test;

#[derive(Deserialize)]
struct Request {
    url: String,
    page_cnt: usize,
}

#[derive(Serialize)]
struct RequestResult {
    found: Vec<(String, Vec<String>)>,
}

fn execute(url: Url, count: usize) -> RequestResult {
    let client = ClientBuilder::new().build().unwrap();
    let link_extractor = LinkExtractor::new(client.clone());
    let flag_extractor =
        FlagExtractor::new(client.clone(), Regex::new(r"MBSD\{[0-9a-zA-Z]+\}").unwrap());
    let crawler = Crawler::new(&link_extractor, url);
    let mut found = Vec::new();

    for url in crawler.take(count) {
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

        let mut texts = Vec::new();

        for node in nodes {
            let text = match node.data {
                Data::Text(text) => text.to_string(),
                _ => continue,
            };

            texts.push(text);
        }

        found.push((url.to_string(), texts));
    }

    RequestResult { found }
}

#[get("/request")]
async fn request(query: web::Query<Request>) -> HttpResponse {
    // TODO: remove adjacent nodes trait
    let res = Arc::new(None);
    let res_clone = Arc::clone(&res);
    let handle = thread::spawn(move || {
        *res_clone = Some(execute(Url::parse(&query.url).unwrap(), query.page_cnt))
    });
    handle.join().unwrap();

    HttpResponse::Ok().json((*res).unwrap())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    HttpServer::new(|| {
        let cors = Cors::default().allow_any_origin();

        App::new()
            .service(request)
            .service(actix_files::Files::new("/", "./public").index_file("index.html"))
            .wrap(cors)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
