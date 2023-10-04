use std::borrow::Borrow;
use std::collections::{HashSet, VecDeque};
use std::hash::Hash;

use log::error;
use regex::Regex;
use reqwest::blocking::Client;
use select::node::{Data, Raw};
use select::{document::Document, predicate::Name};
use thiserror::Error;
use url::{ParseError, Url};

#[derive(Debug, Error)]
pub enum ExtractorError {
    #[error("Failed to send a request")]
    SendRequest(#[source] reqwest::Error),
    #[error("Failed to read the response body")]
    ResponseBody(#[source] reqwest::Error),
    #[error("Failed to make this link URL absolute")]
    AbsolutizeUrl(#[source] url::ParseError),
    #[error("Server returned an error")]
    ServerError(#[source] reqwest::Error),
}

pub struct FlagExtractor {
    client: Client,
    regex: Regex,
}

impl FlagExtractor {
    pub fn new(client: Client, regex: Regex) -> Self {
        Self { client, regex }
    }

    pub fn get_nodes(&self, url: Url) -> Result<Vec<Raw>, ExtractorError> {
        let res = self
            .client
            .get(url)
            .send()
            .map_err(|e| ExtractorError::SendRequest(e))?;
        let res = res
            .error_for_status()
            .map_err(|e| ExtractorError::ServerError(e))?;
        let body = res.text().map_err(|e| ExtractorError::ResponseBody(e))?;
        let doc = Document::from(body.as_str());

        let mut nodes = Vec::new();

        for node in doc.nodes {
            if let Data::Text(ref text) = node.data {
                if self.regex.is_match(&text.to_string()) {
                    nodes.push(node);
                }
            }
        }

        Ok(nodes)
    }
}

pub struct LinkExtractor {
    client: reqwest::Client,
}

impl LinkExtractor {
    pub fn new(client: reqwest::Client) -> Self {
        Self { client }
    }

    pub async fn get_links(&self, url: Url) -> Result<Vec<Url>, ExtractorError> {
        let res = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| ExtractorError::SendRequest(e))?;
        let res = res
            .error_for_status()
            .map_err(|e| ExtractorError::ServerError(e))?;
        let base_url = res.url().clone();
        let body = res
            .text()
            .await
            .map_err(|e| ExtractorError::ResponseBody(e))?;
        let doc = Document::from(body.as_str());
        let mut links = Vec::new();

        for href in doc.find(Name("a")).filter_map(|node| node.attr("href")) {
            let mut url = match Url::parse(href) {
                Ok(url) => url,
                Err(ParseError::RelativeUrlWithoutBase) => base_url
                    .join(href)
                    .map_err(|e| ExtractorError::AbsolutizeUrl(e))?,
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

// impl AdjacentNodes for LinkExtractor {
//     type Node = Url;

//     fn adjacent_nodes(&self, v: &Self::Node) -> Vec<Self::Node> {
//         match self.get_links(v.clone()) {
//             Ok(v) => v,
//             Err(err) => {
//                 error!("{}: {:?}", err, err);
//                 Vec::new()
//             }
//         }
//     }
// }

pub trait AdjacentNodes {
    type Node;

    fn adjacent_nodes(&self, v: &Self::Node) -> Vec<Self::Node>;
}

pub struct Crawler<'a, G: AdjacentNodes> {
    graph: &'a G,
    visit: VecDeque<<G as AdjacentNodes>::Node>,
    visited: HashSet<<G as AdjacentNodes>::Node>,
}

impl<'a, G> Crawler<'a, G>
where
    G: AdjacentNodes,
    <G as AdjacentNodes>::Node: Clone + Hash + Eq + Borrow<<G as AdjacentNodes>::Node>,
{
    pub fn new(graph: &'a G, start: <G as AdjacentNodes>::Node) -> Self {
        let mut visit = VecDeque::new();
        let visited = HashSet::new();

        visit.push_back(start);

        Self {
            graph,
            visit,
            visited,
        }
    }
}

impl<'a, G> Iterator for Crawler<'a, G>
where
    G: AdjacentNodes,
    <G as AdjacentNodes>::Node: Clone + Hash + Eq + Borrow<<G as AdjacentNodes>::Node>,
{
    type Item = <G as AdjacentNodes>::Node;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(v) = self.visit.pop_front() {
            if self.visited.contains(&v) {
                continue;
            }

            let adj = self.graph.adjacent_nodes(&v);
            for u in adj.into_iter() {
                if !self.visited.contains(&u) {
                    self.visit.push_back(u);
                }
            }

            self.visited.insert(v.clone());

            return Some(v);
        }

        None
    }
}
