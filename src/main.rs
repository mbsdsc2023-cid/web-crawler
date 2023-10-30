use actix_cors::Cors;
use actix_web::{get, web, App, HttpResponse, HttpServer};
use crawler::CrawlerResult;
use log::{error, info};
use serde::{Deserialize, Serialize};

use crate::{crawler::Crawler, env::*};

mod crawler;
mod env;

const ADDRESS: &'static str = "127.0.0.1:8080";

#[derive(Debug, Deserialize)]
struct Request {
    url: String,
}

#[derive(Debug, Serialize)]
struct PageResult {
    url: String,
    matched_strings: Vec<String>,
}

#[derive(Debug, Serialize)]
struct Result {
    crawler_results: Vec<CrawlerResult>,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum RequestResult {
    Ok(Result),
    Error(String),
}

#[get("/request")]
async fn request(query: web::Query<Request>) -> HttpResponse {
    info!("{:?}", query.0);

    let crawler = match Crawler::new(r"MBSD\{[0-9a-zA-Z]+\}", &query.url) {
        Ok(c) => c,
        Err(e) => {
            error!("{}: {:?}", e, e);
            return HttpResponse::Ok().json(RequestResult::Error(e.to_string()));
        }
    };

    let mut results = match crawler.execute().await {
        Ok(res) => res,
        Err(e) => {
            error!("{}: {:?}", e, e);
            return HttpResponse::Ok().json(RequestResult::Error(e.to_string()));
        }
    };

    // exclude xml file
    results.retain(|r| !r.url.as_str().contains(".xml"));

    for r in results.iter() {
        info!("{:?}", r);
    }

    HttpResponse::Ok().json(RequestResult::Ok(Result {
        crawler_results: results,
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    info!("{} - {}", ENV_NAME, ENV_DESCRIPTION);
    info!("Version: {}", ENV_VERSION);

    let server = HttpServer::new(|| {
        let cors = Cors::default().allow_any_origin();

        App::new()
            .service(request)
            .service(actix_files::Files::new("/", "./public").index_file("index.html"))
            .wrap(cors)
    })
    .bind(ADDRESS)?;

    info!("Running web crawler at {}", ADDRESS);
    server.run().await
}
