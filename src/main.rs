use anyhow::Context;
use graphql_client::reqwest::post_graphql;
use moxie_stuff::{
    check_claim_transaction_status, check_user_everyday_rewards_amount, claim_everyday_rewards,
    https_client, portfolio_tokens,
};
use reqwest::header::HeaderMap;
use tracing::info;

#[derive(serde::Deserialize)]
struct Config {
    airstack_api_key: String,
    farcaster_id: i64,
    /// TODO: Address type.
    preferred_connected_wallet: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().unwrap();

    let subscriber = tracing_subscriber::fmt().pretty().finish();
    // TODO: tokio-console subscriber, too
    tracing::subscriber::set_global_default(subscriber)?;

    let config: Config = envy::from_env()?;

    info!(
        "Hello, #{} ({})!",
        config.farcaster_id, config.preferred_connected_wallet
    );

    let mut airstack_claims_headers = HeaderMap::new();
    airstack_claims_headers.insert(
        "x-airstack-claims",
        config.airstack_api_key.parse().unwrap(),
    );
    let airstack_claims_url = "https://claims.airstack.xyz/moxie";
    let airstack_protocol_subsgraph_url = "https://airstack.xyz/api/protocol-subgraph";

    let airstack_client = https_client(airstack_claims_headers)?;

    // TODO: check if the user has Moxie available to claim
    let check_result = post_graphql::<check_user_everyday_rewards_amount::Query, _>(
        &airstack_client,
        airstack_claims_url,
        check_user_everyday_rewards_amount::Variables {
            fid: config.farcaster_id,
        },
    )
    .await?
    .data
    .context("no data in claim result")?
    .farcaster_user_claim_transaction_details;

    if check_result.available_claim_amount_in_wei == "0" {
        info!("No Moxie available to claim");
        return Ok(());
    }

    // TODO: check all the fan tokens that the user owns. we want to buy more fan tokens to keep the same ratios

    let balances = post_graphql::<portfolio_tokens::Query, _>(
        &airstack_client,
        airstack_protocol_subsgraph_url,
        portfolio_tokens::Variables {
            limit: 1000,
            wallet_addresses: vec![config.preferred_connected_wallet.clone()],
            skip: None,
        },
    )
    .await?
    .data
    .context("no data")?
    .users;

    let claim_result = post_graphql::<claim_everyday_rewards::Query, _>(
        &airstack_client,
        airstack_claims_url,
        claim_everyday_rewards::Variables {
            fid: config.farcaster_id,
            preferred_connected_wallet: config.preferred_connected_wallet,
        },
    )
    .await?
    .data
    .context("no data in claim result")?
    .farcaster_user_claim_moxie
    .context("no inner data in claim result")?;

    let claim_transaction_id = claim_result.transaction_id.unwrap();

    // TODO: poll the transaction status until it is successful

    // claim_wei = transaction_result.transaction_amount_in_wei.unwrap();

    // TODO: buy fan tokens with the claimed moxie

    Ok(())
}
