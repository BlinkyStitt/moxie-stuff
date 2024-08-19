use graphql_client::reqwest::post_graphql;
use moxie_stuff::{
    claim_everyday_rewards::{farcaster_user_claim_moxie, FarcasterUserClaimMoxie},
    https_client,
};
use reqwest::header::HeaderMap;

#[derive(serde::Deserialize)]
struct Config {
    airstack_api_key: String,
    farcaster_id: i64,
    /// TODO: Address type.
    connected_wallet: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().unwrap();

    let subscriber = tracing_subscriber::fmt().pretty().finish();
    // TODO: tokio-console subscriber, too
    tracing::subscriber::set_global_default(subscriber)?;

    tracing::info!("Hello, world!");

    let config: Config = envy::from_env()?;

    let mut airstack_claims_headers = HeaderMap::new();
    airstack_claims_headers.insert(
        "x-airstack-claims",
        config.airstack_api_key.parse().unwrap(),
    );

    let airstack_client = https_client(airstack_claims_headers)?;

    // TODO: check if the user has Moxie available to claim

    // TODO: check all the fan tokens that the user owns. select the top 50%.

    let claim_variables = farcaster_user_claim_moxie::Variables {
        fid: config.farcaster_id,
        preferred_connected_wallet: config.connected_wallet,
    };

    let claim_result = post_graphql::<FarcasterUserClaimMoxie, _>(
        &airstack_client,
        "https://claims.airstack.xyz/moxie",
        claim_variables,
    )
    .await?;

    let claim_result = claim_result
        .data
        .unwrap()
        .farcaster_user_claim_moxie
        .unwrap();

    let claim_transaction_id = claim_result.transaction_id.unwrap();

    // TODO: poll the transaction status until it is successful

    // claim_wei = transaction_result.transaction_amount_in_wei.unwrap();

    // TODO: buy fan tokens with the claimed moxie

    Ok(())
}
