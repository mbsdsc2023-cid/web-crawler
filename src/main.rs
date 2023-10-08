use std::env;

use crate::crawler::LinkExtractor;

use actix_cors::Cors;
use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use log::{error, info};
use reqwest::ClientBuilder;
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
    links: Vec<String>,
}

#[get("/request")]
async fn request(index: web::Query<Request>) -> HttpResponse {
    // TODO extractor not must be async method?
    let url = Url::parse(&index.url).unwrap();
    let client = ClientBuilder::new().build().unwrap();
    let link_extractor = LinkExtractor::new(client);
    match link_extractor.get_links(url).await {
        Ok(links) => {
            for l in links.iter() {
                println!("{}", l);
            }
            HttpResponse::Ok().json(RequestResult {
                links: links.iter().map(|l| l.to_string()).collect(),
            })
        }
        Err(err) => {
            error!("{}", err);
            HttpResponse::Ok().json(RequestResult { links: vec![] })
        }
    }
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
