//! `st` - A CLI for managing stacked PRs locally and on GitHub.
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![allow(clippy::result_large_err)]

use clap::Parser;

mod ai;
mod cli;
mod config;
mod constants;
mod ctx;
mod errors;
mod git;
mod subcommands;
mod tree;

#[tokio::main]
async fn main() {
    if let Err(e) = cli::Cli::parse().run().await {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}
