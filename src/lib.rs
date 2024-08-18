mod graphql;

use std::time::Duration;

pub use graphql::*;
use reqwest::header::{HeaderMap, HeaderValue};

/// The application name and version.
pub static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

/// create a new HTTPS-only client with our app's user agent.
pub fn https_client(default_headers: HeaderMap<HeaderValue>) -> reqwest::Result<reqwest::Client> {
    let client = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .http2_keep_alive_interval(Duration::from_secs(50))
        .http2_keep_alive_timeout(Duration::from_secs(60))
        .https_only(true)
        .user_agent(APP_USER_AGENT)
        .default_headers(default_headers)
        .build()?;

    Ok(client)
}
