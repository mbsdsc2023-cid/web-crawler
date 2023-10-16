use std::env;

use actix_cors::Cors;
use actix_web::{get, web, App, HttpResponse, HttpServer};
use log::{error, info};
use serde::{Deserialize, Serialize};

use crate::crawler::Crawler;

mod crawler;
mod test;

const ADDRESS: &'static str = "127.0.0.1:8080";

#[derive(Debug, Deserialize)]
struct Request {
    url: String,
}

#[derive(Debug, Serialize)]
struct Result {
    url: String,
    matched_strings: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum RequestResult {
    Ok(Vec<Result>),
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

    let res = match crawler.execute().await {
        Ok(res) => res,
        Err(e) => {
            error!("{}: {:?}", e, e);
            return HttpResponse::Ok().json(RequestResult::Error(e.to_string()));
        }
    };

    let mut results = Vec::new();
    for (url, matched_strings) in res {
        if url.as_str().contains(".xml") {
            continue;
        }

        let result = Result {
            url: url.to_string(),
            matched_strings,
        };

        info!("{:?}", result);
        results.push(result);
    }

    HttpResponse::Ok().json(RequestResult::Ok(results))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env::set_var("RUST_LOG", "info");
    env_logger::init();

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
