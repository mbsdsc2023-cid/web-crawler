use clap::Parser;

#[derive(Debug, Parser)]
pub struct Args {
    pub url: String,
    pub pages: usize,
}
