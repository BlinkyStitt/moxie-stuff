mod captcha;
mod frame_crawler;
mod frames;
mod graphql;
mod interfaces;

use alloy::primitives::Address;
use reqwest::header::{HeaderMap, HeaderValue};
use std::time::Duration;

pub use captcha::*;
pub use frame_crawler::*;
pub use frames::*;
pub use graphql::*;
pub use interfaces::*;

/// The application name and version.
pub static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

/// create a new HTTPS-only client with our app's user agent.
pub fn https_client(default_headers: HeaderMap<HeaderValue>) -> reqwest::Result<reqwest::Client> {
    let client = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(5))
        .timeout(Duration::from_secs(50))
        .http2_keep_alive_interval(Duration::from_secs(50))
        .http2_keep_alive_timeout(Duration::from_secs(60))
        .https_only(true)
        .user_agent(APP_USER_AGENT)
        .default_headers(default_headers)
        .build()?;

    Ok(client)
}

/// Using this requires an [approved API key](https://forms.gle/th7hKumcxz3X5txZ6).
pub fn airstack_claims_client(api_key: &str) -> anyhow::Result<reqwest::Client> {
    let mut default_headers = HeaderMap::new();
    default_headers.insert("x-airstack-claims", api_key.parse()?);

    let client = https_client(default_headers)?;

    Ok(client)
}

pub fn neynar_client(api_key: &str) -> anyhow::Result<reqwest::Client> {
    let mut default_headers = HeaderMap::new();
    default_headers.insert("api_key", api_key.parse()?);

    let client = https_client(default_headers)?;

    Ok(client)
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AirstackConnectedAddresses {
    pub beneficiary_address: Address,
    pub vesting_contract_address: Address,
    pub custody_address: Address,
}

impl AirstackConnectedAddresses {
    pub fn to_vec(&self) -> Vec<String> {
        vec![
            self.beneficiary_address.to_string(),
            self.vesting_contract_address.to_string(),
            self.custody_address.to_string(),
        ]
    }
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AirstackConnectedAddressesRequest {
    /// TODO: yes. this is a String, not an i64. we'll see how that goes
    pub fid: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AirstackConnectedAddressesResponse {
    pub addresses: AirstackConnectedAddresses,
}

pub async fn airstack_connected_addresses(
    client: &reqwest::Client,
    fid: i64,
) -> anyhow::Result<AirstackConnectedAddresses> {
    let request = AirstackConnectedAddressesRequest {
        fid: fid.to_string(),
    };

    let response = client
        .post("https://airstack.xyz/api/connected-addresses")
        .header("Origin", "https://airstack.xyz")
        .json(&request)
        .send()
        .await?
        .error_for_status()?
        .json::<AirstackConnectedAddressesResponse>()
        .await?;

    Ok(response.addresses)
}
