#![allow(dead_code)]

pub mod auth;
pub mod user;

pub use anyhow::Result;
pub use reqwest_middleware::ClientWithMiddleware as Client;
