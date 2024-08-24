use anyhow::Context;
use moxie_stuff::{
    airstack_claims_client, airstack_connected_addresses, check_claim_transaction_status,
    check_user_everyday_rewards_amount, claim_everyday_rewards, get_nota_stats, https_client,
    portfolio_tokens,
};
use tracing::{error, info};

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

    info!("connected_wallets: {:#?}", connected_addresses);

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

    // check if the user has Moxie available to claim.
    // TODO: something is wrong cuz this response is just empty.
    // TODO: if no Moxie API key, use Neynar's frame crawler to view our balance
    let check_result = check_user_everyday_rewards_amount::send_request(
        &airstack_claims_client,
        check_user_everyday_rewards_amount::Variables {
            fid: config.farcaster_id,
        },
    )
    .await?;

    info!("check_result: {:#?}", check_result);

    if let Some(check_result) = check_result.data {
        let check_result = check_result.farcaster_user_claim_transaction_details;

        if check_result.available_claim_amount_in_wei == "0" {
            info!("No Moxie available to claim");
            return Ok(());
        }

        // TODO: environment variable to skip the claim step

        // let claim_result = post_graphql::<Query::send_request(
        //     &airstack_claims_client,
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
    } else {
        error!("API key is not allowed. Fill out <https://forms.gle/th7hKumcxz3X5txZ6> and then check your email. It might take a few days to get approved.");

        // TODO: share a link to the form

        // TODO: share a link to a claim frame so they can claim themselves

        // TODO: use Neynar API to click the button in the frame so that you don't need to be on their allow list
    }

    // TODO: buy fan tokens with the loose moxie (subtract a configurable "slush fund" amount to leave some tokens for the user to spend manually)

    Ok(())
}
