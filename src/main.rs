use anyhow::Context;
use graphql_client::reqwest::post_graphql;
use moxie_stuff::{
    airstack_claims_client, airstack_connected_addresses, check_claim_transaction_status,
    check_user_everyday_rewards_amount, claim_everyday_rewards, get_nota_stats, https_client,
    portfolio_tokens, AIRSTACK_CLAIMS_URL, AIRSTACK_PROTOCOL_SUBGRAPH_URL,
};
use reqwest::header::HeaderMap;
use tracing::info;

#[derive(serde::Deserialize)]
struct Config {
    airstack_api_key: String,
    farcaster_id: i64,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().unwrap();

    let subscriber = tracing_subscriber::fmt().pretty().finish();
    // TODO: tokio-console subscriber, too
    tracing::subscriber::set_global_default(subscriber)?;

    let config: Config = envy::from_env()?;

    info!("Hello, #{}!", config.farcaster_id);

    let https_client = https_client(Default::default())?;
    let airstack_claims_client = airstack_claims_client(&config.airstack_api_key)?;

    let connected_addresses =
        airstack_connected_addresses(&https_client, config.farcaster_id).await?;

    info!("connected_wallets: {:#?}", connected_addresses.to_vec());

    /*

    // check if the user has Moxie available to claim.
    // TODO: something is wrong cuz this response is just empty.
    let check_result = post_graphql::<check_user_everyday_rewards_amount::Query, _>(
        &airstack_claims_client,
        check_user_everyday_rewards_amount::URL,
        check_user_everyday_rewards_amount::Variables {
            fid: config.farcaster_id,
        },
    )
    .await?;

    info!("check_result: {:#?}", check_result);

    let check_result = check_result
        .data
        .context("no data in claim result")?
        .farcaster_user_claim_transaction_details;

    if check_result.available_claim_amount_in_wei == "0" {
        info!("No Moxie available to claim");
        return Ok(());
    }
    */

    // TODO: check all the fan tokens that the user owns. we want to buy more fan tokens to keep the same ratios

    // TODO: get connected_wallets from the subgraph

    let balances = post_graphql::<portfolio_tokens::Query, _>(
        &https_client,
        portfolio_tokens::URL,
        portfolio_tokens::Variables {
            limit: 1000,
            wallet_addresses: connected_addresses.to_vec(),
            skip: None,
        },
    )
    .await?
    .data
    .context("no data")?
    .users;

    info!("Balances: {:#?}", balances);

    // TODO: fetch stats for all our tokens
    let coopa_stats = post_graphql::<get_nota_stats::Query, _>(
        &https_client,
        get_nota_stats::URL,
        get_nota_stats::Variables {
            fid_or_channel_url: "206".to_string(),
        },
    )
    .await?
    .data
    .context("no data")?;
    info!("Coopa stats: {:#?}", coopa_stats);

    // let claim_result = post_graphql::<claim_everyday_rewards::Query, _>(
    //     &airstack_claims_client,
    //     claim_everyday_rewards::URL,
    //     claim_everyday_rewards::Variables {
    //         fid: config.farcaster_id,
    //         preferred_connected_wallet: connected_addresses.beneficiary_address.clone(),
    //     },
    // )
    // .await?
    // .data
    // .context("no data in claim result")?
    // .farcaster_user_claim_moxie
    // .context("no inner data in claim result")?;

    // let claim_transaction_id = claim_result.transaction_id.unwrap();

    // TODO: poll the transaction status until it is successful

    // claim_wei = transaction_result.transaction_amount_in_wei.unwrap();

    // TODO: buy fan tokens with the claimed moxie

    Ok(())
}
