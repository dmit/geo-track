#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! This crate contains the different components of the `geo-track` backend
//! service.

pub mod http;
pub mod ingest;
pub mod storage;
