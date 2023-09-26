use reqwest::blocking::Client;
use select::{document::Document, predicate::Name};
use thiserror::Error;
use url::{ParseError, Url};

#[derive(Debug, Error)]
pub enum GetLinksError {
    #[error("Failed to send a request")]
    SendRequest(#[source] reqwest::Error),
    #[error("Failed to read the response body")]
    ResponseBody(#[source] reqwest::Error),
    #[error("Failed to make this link URL absolute")]
    AbsolutizeUrl(#[source] url::ParseError),
    #[error("Server returned an error")]
    ServerError(#[source] reqwest::Error),
}

pub struct LinkExtractor {
    client: Client,
}

impl LinkExtractor {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub fn get_links(&self, url: Url) -> Result<Vec<Url>, GetLinksError> {
        let res = self
            .client
            .get(url)
            .send()
            .map_err(|e| GetLinksError::SendRequest(e))?;
        let res = res
            .error_for_status()
            .map_err(|e| GetLinksError::ServerError(e))?;
        let base_url = res.url().clone();
        let body = res.text().map_err(|e| GetLinksError::ResponseBody(e))?;
        let doc = Document::from(body.as_str());
        let mut links = Vec::new();

        for href in doc.find(Name("a")).filter_map(|node| node.attr("href")) {
            let mut url = match Url::parse(href) {
                Ok(url) => url,
                Err(ParseError::RelativeUrlWithoutBase) => base_url
                    .join(href)
                    .map_err(|e| GetLinksError::AbsolutizeUrl(e))?,
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
}
