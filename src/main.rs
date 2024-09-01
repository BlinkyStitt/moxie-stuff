use anyhow::Context;
use moxie_stuff::{
    airstack_claims_client, airstack_connected_addresses, check_claim_transaction_status,
    check_user_everyday_rewards_amount, claim_everyday_rewards, claim_everyday_rewards_with_neynar,
    get_nota_stats, https_client, portfolio_tokens, FrameCrawler,
};
use tracing::{error, info};

#[derive(serde::Deserialize)]
struct Config {
    airstack_api_key: String,
    farcaster_id: i64,
    neynar_api_key: String,
    neynar_signer_uuid: String,
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

    info!("connected_wallets: {:#?}", connected_addresses);

    let frame_crawler = FrameCrawler::new(
        connected_addresses.beneficiary_address.clone(),
        config.neynar_api_key,
        config.neynar_signer_uuid,
    )
    .await?;

    // TODO: make this optional. only claim if over a certain threshold.
    claim_everyday_rewards_with_neynar(&frame_crawler).await?;

    let balances = portfolio_tokens::send_request(
        &https_client,
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
    let my_stats = get_nota_stats::send_request(
        &https_client,
        get_nota_stats::Variables {
            fid_or_channel_url: config.farcaster_id.to_string(),
        },
    )
    .await?
    .data
    .context("no data")?;
    info!("Fid {} stats: {:#?}", config.farcaster_id, my_stats);

    // TODO: check if we have any funds available to claim. i guess we have to read the moxie frame's response

    // // TODO: use Neynar API to click the button in the frame so that you don't need to be on their allow list
    // claim_everyday_rewards_with_neynar(&frame_crawler).await?;

    // TODO: buy fan tokens with the loose moxie (subtract a configurable "slush fund" amount to leave some tokens for the user to spend manually)

    Ok(())
}
