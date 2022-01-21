#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]
#![deny(
    missing_debug_implementations,
    clippy::print_stderr,
    clippy::print_stdout
)]

//! Core data structures of the library

pub mod entities;
pub mod lang;
pub mod mutate;
pub mod query;
pub mod schema;

pub use sea_orm;
pub use sea_query;
