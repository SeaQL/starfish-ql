#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]
#![deny(
    missing_debug_implementations,
    clippy::print_stderr,
    clippy::print_stdout
)]

//! A graph database and query engine
//!
//! Copyright (c) 2021 Tsang Hao Fung

pub mod core;
pub mod mutate;
pub mod query;
pub mod schema;
