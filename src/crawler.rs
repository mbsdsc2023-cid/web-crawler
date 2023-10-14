use std::collections::HashSet;

use regex::Regex;
use reqwest::{Client, ClientBuilder};
use select::{
    document::Document,
    node::{Data, Raw},
    predicate::Name,
};
use thiserror::Error;
use url::{ParseError, Url};

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

    pub async fn execute(&self) -> Result<Vec<(Url, Vec<String>)>, CrawlerError> {
        let mut queue = vec![self.base_url.clone()];
        let mut visited = HashSet::new();
        let mut result = Vec::new();

        while let Some(url) = queue.pop() {
            if visited.contains(&url) {
                continue;
            }

            let links = self.get_links(&url).await?;
            for link in links {
                if !visited.contains(&link) {
                    queue.push(link);
                }
            }

            let matched_nodes = self.get_matched_nodes(&url).await?;
            let mut matched_strings = Vec::new();
            for node in matched_nodes {
                match node.data {
                    Data::Text(ref text) => matched_strings.push(text.to_string()),
                    Data::Comment(ref text) => matched_strings.push(text.to_string()),
                    Data::Element(_, quals) => {
                        for (_, ref text) in quals {
                            let text = text.to_string();
                            if self.target_regex.is_match(&text) {
                                matched_strings.push(text);
                            }
                        }
                    }
                };
            }

            if matched_strings.len() > 0 {
                result.push((url.clone(), matched_strings));
            }

            visited.insert(url);
        }

        Ok(result)
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

    async fn get_matched_nodes(&self, url: &Url) -> Result<Vec<Raw>, CrawlerError> {
        let doc = self.get_doc(url.clone()).await?;
        let mut nodes = Vec::new();

        for node in doc.nodes {
            match &node.data {
                Data::Text(ref text) => {
                    if self.target_regex.is_match(&text.to_string()) {
                        nodes.push(node.clone());
                    }
                }
                Data::Comment(ref text) => {
                    if self.target_regex.is_match(&text.to_string()) {
                        nodes.push(node.clone());
                    }
                }
                Data::Element(_, quals) => {
                    for (_, ref text) in quals {
                        if self.target_regex.is_match(&text.to_string()) {
                            nodes.push(node.clone());
                            break;
                        }
                    }
                }
            }
        }

        Ok(nodes)
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
        let body = res
            .text()
            .await
            .map_err(|e| CrawlerError::ResponseBody(e))?;

        Ok(Document::from(body.as_str()))
    }
}
