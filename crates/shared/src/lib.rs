//! This crate houses base data structures that can be used by different
//! backend and frontend services, as well as encoding data in transit and at
//! rest.
//!
//! The crate is marked `no_std`, which makes it possible to use it even on
//! small embedded devices.

#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod data;
