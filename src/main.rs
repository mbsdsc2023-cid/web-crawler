use std::env;

use crate::crawler::LinkExtractor;

use actix_cors::Cors;
use actix_web::{get, web, App, HttpServer, Responder};
use log::{error, info};
use reqwest::ClientBuilder;
use serde::Deserialize;
use url::Url;

mod cli;
mod crawler;
mod test;

// TODO: exits other way?
const DASHBOARD_HTML: &'static str = include_str!("../dashboard.html");

#[derive(Deserialize)]
struct Index {
    url: String,
    page_cnt: usize,
}

// TODO
#[get("/dashboard.html")]
async fn dashboard() -> impl Responder {
    DASHBOARD_HTML
}

#[get("/")]
async fn index(index: web::Query<Index>) -> impl Responder {
    // TODO extractor not must be async method?
    let url = Url::parse(&index.url).unwrap();
    let client = ClientBuilder::new().build().unwrap();
    let link_extractor = LinkExtractor::new(client);
    match link_extractor.get_links(url).await {
        Ok(links) => {
            for l in links {
                println!("{}", l);
            }
        }
        Err(err) => error!("{}", err),
    }

    // TODO: return json
    format!("url: {}, count: {}", index.url, index.page_cnt)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    HttpServer::new(|| {
        let cors = Cors::default().allow_any_origin();
        App::new().wrap(cors).service(index)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
