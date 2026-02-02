//! `st` - A CLI for managing stacked PRs locally and on GitHub.
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![allow(clippy::result_large_err)]
#![allow(dead_code)] // Most code is used by the binary, not the library
#![allow(unused_crate_dependencies)] // Dependencies are used by binary

pub mod config;
pub mod constants;
pub mod errors;
pub mod tree;

// Internal modules (used by binary)
mod ai;
mod cli;
mod ctx;
mod git;
mod subcommands;
