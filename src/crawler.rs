use std::{collections::HashSet, time::Instant};

use regex::Regex;
use reqwest::{Client, ClientBuilder};
use select::{document::Document, node::Data, predicate::Name};
use serde::Serialize;
use thiserror::Error;
use url::{ParseError, Url};

#[derive(Debug, Serialize)]
pub struct CrawlerResult {
    pub url: String,
    pub page_title: String,
    pub targets: Vec<String>,
    pub elapsed_ms: u128,
}

#[derive(Debug, Error)]
pub enum CrawlerError {
    #[error("Failed to build client")]
    ClientBuilder(#[source] reqwest::Error),
    #[error("Failed to create regex")]
    RegexError(#[source] regex::Error),
    #[error("Failed to send a request")]
    SendRequest(#[source] reqwest::Error),
    #[error("Server returned an error")]
    ServerError(#[source] reqwest::Error),
    #[error("Failed to read the response body")]
    ResponseBody(#[source] reqwest::Error),
    #[error("Failed to parse URL")]
    ParseUrl(#[source] url::ParseError),
}

pub struct Crawler {
    client: Client,
    target_regex: Regex,
    base_url: Url,
}

impl Crawler {
    pub fn new(target_regex: &str, base_url: &str) -> Result<Self, CrawlerError> {
        let client = ClientBuilder::new()
            .build()
            .map_err(|e| CrawlerError::ClientBuilder(e))?;
        let target_regex = Regex::new(target_regex).map_err(|e| CrawlerError::RegexError(e))?;
        let base_url = Url::parse(base_url).map_err(|e| CrawlerError::ParseUrl(e))?;

        Ok(Self {
            client,
            target_regex,
            base_url,
        })
    }

    pub async fn execute(&self) -> Result<Vec<CrawlerResult>, CrawlerError> {
        let mut queue = vec![self.base_url.clone()];
        let mut visited = HashSet::new();
        let mut results = Vec::new();

        while let Some(url) = queue.pop() {
            if visited.contains(&url) {
                continue;
            }

            let start = Instant::now();
            let links = self.get_links(&url).await?;
            for link in links {
                if !visited.contains(&link) {
                    queue.push(link);
                }
            }

            let doc = self.get_doc(url.clone()).await?;
            let matched_strings = self.get_matched_strings(&doc).await?;
            let page_title = match self.get_doc_title(&doc).await {
                Some(title) => title,
                None => "".to_string(),
            };

            let end = start.elapsed();

            if matched_strings.len() > 0 {
                let result = CrawlerResult {
                    url: url.to_string(),
                    page_title,
                    targets: matched_strings,
                    elapsed_ms: end.as_millis(),
                };
                results.push(result);
            }

            visited.insert(url);
        }

        Ok(results)
    }

    async fn get_links(&self, url: &Url) -> Result<Vec<Url>, CrawlerError> {
        let base_url = url.clone();
        let doc = self.get_doc(url.clone()).await?;
        let mut links = Vec::new();

        for href in doc.find(Name("a")).filter_map(|node| node.attr("href")) {
            let mut url = match Url::parse(href) {
                Ok(url) => url,
                Err(ParseError::RelativeUrlWithoutBase) => {
                    base_url.join(href).map_err(|e| CrawlerError::ParseUrl(e))?
                }
                _ => continue,
            };
            url.set_fragment(None);

            if !url.as_str().contains(base_url.as_str()) {
                continue;
            }

            links.push(url);
        }

        Ok(links)
    }

    async fn get_matched_strings(&self, doc: &Document) -> Result<Vec<String>, CrawlerError> {
        let mut matched_strings = Vec::new();
        for node in &doc.nodes {
            let strings = match &node.data {
                Data::Text(text) | Data::Comment(text) => vec![text.to_string()],
                Data::Element(_, quals) => quals.iter().map(|(_, text)| text.to_string()).collect(),
            };

            matched_strings.extend(strings.iter().flat_map(|s| {
                self.target_regex
                    .find_iter(s)
                    .map(|mat| mat.as_str().to_string())
            }));
        }

        Ok(matched_strings)
    }

    async fn get_doc_title(&self, doc: &Document) -> Option<String> {
        let titles: Vec<&str> = doc
            .find(Name("title"))
            .filter_map(|node| node.first_child())
            .filter_map(|node| node.as_text())
            .collect();

        if titles.len() != 1 {
            return None;
        }

        Some(titles[0].to_string())
    }

    async fn get_doc(&self, url: Url) -> Result<Document, CrawlerError> {
        let res = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| CrawlerError::SendRequest(e))?;
        let res = res
            .error_for_status()
            .map_err(|e| CrawlerError::ServerError(e))?;

        let body = &res
            .text()
            .await
            .map_err(|e| CrawlerError::ResponseBody(e))?;

        Ok(Document::from(body.as_str()))
    }
}
