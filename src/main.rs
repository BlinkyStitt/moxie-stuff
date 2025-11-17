use std::{collections::HashMap, str::FromStr};

use alloy::{
    network::EthereumWallet,
    primitives::{address, Address, U256},
    providers::{Provider, ProviderBuilder},
    signers::local::PrivateKeySigner,
};
use bigdecimal::{BigDecimal, ToPrimitive};
use chrono::Utc;
use eyre::{Context, ContextCompat};
use moxie_stuff::{
    airstack_connected_addresses, claim_everyday_rewards_with_neynar, get_ftas_by_symbol,
    https_client, portfolio_tokens, FrameCrawler, IUniswapV2Router02, MoxieBondingCurve,
    MoxieToken, NeynarError, USDC,
};
use num_bigint::BigInt;
use num_traits::{FromPrimitive, Zero};
use tracing::{debug, info};
use url::Url;

#[derive(serde::Deserialize)]
struct Config {
    base_rpc_url: Url,
    base_address: Address,
    base_private_key: String,
    farcaster_id: i64,
    neynar_api_key: String,
    neynar_signer_uuid: String,
}

/// TODO: builder pattern with bon for creating this?
struct SubjectTokenData<'a> {
    symbol: &'a str,
    id: &'a str,
    display_name: Option<String>,
    current_price_in_moxie: BigDecimal,
    total_supply_wei: BigDecimal,
    balance_wei: BigDecimal,
    avg_daily_earnings: BigDecimal,
    earnings_this_week: BigDecimal,
    user_fans_share_percentage: Option<BigDecimal>,
}

impl SubjectTokenData<'_> {
    /// TODO: i think this apr calc is wrong
    fn apr(&self) -> BigDecimal {
        self.avg_daily_rewards_per_moxie()
            * BigDecimal::from_u64(365).unwrap()
            * BigDecimal::from_u64(100).unwrap()
    }

    fn avg_daily_rewards_per_moxie(&self) -> BigDecimal {
        // TODO: &self.avg_daily_earnings / &self.tvl? or tvl/avg?
        &self.avg_daily_rewards_per_fan_token() / &self.current_price_in_moxie
    }

    fn avg_fan_daily_rewards_per_moxie(&self) -> BigDecimal {
        let mut x = self.avg_daily_rewards_per_moxie();

        if let Some(user_fans_share_percentage) = &self.user_fans_share_percentage {
            x *= user_fans_share_percentage / BigDecimal::from_u64(100).unwrap();
        }

        x
    }

    /// TODO: this feels wrong. i think we want to use TVL here instead?
    /// TODO: SEE: <https://mirror.xyz/siddxa.eth/wgGqLMmYzKhZtBVBPPNPyJaxNF_GIJAo2MwlFMV4Hvo>
    /// TODO: need to include the `userFansSharePercentage` in the calculation
    fn avg_daily_rewards_per_fan_token(&self) -> BigDecimal {
        &self.avg_daily_earnings / &self.total_tokens()
    }

    fn balance(&self) -> BigDecimal {
        &self.balance_wei / BigDecimal::from_str("1e18").unwrap()
    }

    fn total_tokens(&self) -> BigDecimal {
        &self.total_supply_wei / BigDecimal::from_str("1e18").unwrap()
    }
}

impl std::fmt::Debug for SubjectTokenData<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SubjectTokenData")
            .field("symbol", &self.symbol)
            .field("id", &self.id)
            .field("display_name", &self.display_name)
            .field(
                "current_price_in_moxie",
                &self.current_price_in_moxie.to_f32(),
            )
            .field("total_tokens", &self.total_tokens().to_f32())
            .field("balance", &self.balance().to_f32())
            .field("avg_daily_earnings", &self.avg_daily_earnings.to_f32())
            .field("earnings_this_week", &self.earnings_this_week.to_f32())
            .field(
                "avg_daily_rewards_per_moxie",
                &self.avg_daily_rewards_per_moxie().to_f32(),
            )
            .field(
                "avg_daily_rewards_per_fan_token",
                &self.avg_daily_rewards_per_fan_token().to_f32(),
            )
            .field(
                "user_fans_share_percentage",
                &self.user_fans_share_percentage.as_ref().map(|x| x.to_f32()),
            )
            .field(
                "avg_fan_daily_rewards_per_moxie",
                &self.avg_fan_daily_rewards_per_moxie().to_f32(),
            )
            .finish()
    }
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    dotenvy::dotenv().wrap_err(".env is required for setting credentials")?;

    let subscriber = tracing_subscriber::fmt().pretty().finish();
    // TODO: tokio-console subscriber, too
    tracing::subscriber::set_global_default(subscriber)?;

    let config: Config = envy::from_env()?;

    // TODO: get the farcaster id from the neynar api?
    info!("Hello, #{}!", config.farcaster_id);

    let https_client = https_client(Default::default())?;
    // let airstack_claims_client = airstack_claims_client(&config.airstack_api_key)?;

    let connected_addresses =
        airstack_connected_addresses(&https_client, config.farcaster_id).await?;

    info!("connected_wallets: {:#?}", connected_addresses);

    let signer: PrivateKeySigner = config.base_private_key.parse()?;

    eyre::ensure!(
        signer.address() == config.base_address,
        "wallet address does not match config"
    );

    let wallet = EthereumWallet::from(signer);

    info!("Wallet: {:#?}", wallet);

    let base_provider = ProviderBuilder::new()
        .with_recommended_fillers()
        .wallet(wallet)
        // .with_chain(NamedChain::Base)
        .on_http(config.base_rpc_url);

    let latest_block = base_provider.get_block_number().await?;

    // Print the block number.
    info!("Latest block number: {latest_block}");

    let moxie_bonding_curve = MoxieBondingCurve::new(
        address!("02bDb83bE769351771dCdd30b51c141b0Bd5FF41"),
        base_provider.clone(),
    );

    let moxie_token_address = moxie_bonding_curve.token().call().await?._0;

    let moxie_token = MoxieToken::new(moxie_token_address, base_provider.clone());

    let usdc = USDC::new(
        address!("833589fcd6edb6e08f4c7c32d4f71b54bda02913"),
        base_provider.clone(),
    );

    let uniswap_v2_router = IUniswapV2Router02::new(
        address!("4752ba5dbc23f44d87826276bf6fd6b1c372ad24"),
        base_provider.clone(),
    );

    let frame_crawler = FrameCrawler::new(
        connected_addresses.beneficiary_address,
        config.neynar_api_key,
        config.neynar_signer_uuid,
    )
    .await?;

    // TODO: make this optional. only claim if over a certain threshold.
    // TODO: better error types so that we can skip if we get an "already claimed"/"no claim button" error
    match claim_everyday_rewards_with_neynar(&frame_crawler).await {
        Ok(_) => {
            // we claimed. sell half

            // // TODO: think more about this reset
            // base_provider
            //     .anvil_reset(Some(Forking {
            //         json_rpc_url: None,
            //         block_number: None,
            //     }))
            //     .await?;

            let moxie_balance_wei = moxie_token
                .balanceOf(connected_addresses.beneficiary_address)
                .call()
                .await?
                ._0;

            info!("moxie_balance_wei: {}", moxie_balance_wei);

            // TODO: save this amount in a database so that we can resume if we error
            let sell_moxie_wei = moxie_balance_wei / U256::from(2);

            info!("sell_moxie_wei: {}", sell_moxie_wei);

            let path = vec![*moxie_token.address(), *usdc.address()];

            // TODO: move this sell logic into a helper function that compares multiple exchanges
            // TODO: use https://swap.defillama.com/ to find the best price
            let amounts_out = uniswap_v2_router
                .getAmountsOut(sell_moxie_wei, path.clone())
                .call()
                .await?
                .amounts;

            let amount_out = amounts_out.last().unwrap();

            // TODO: apply slippage to amount_out

            let min_amount_out_wei = amount_out * U256::from(995) / U256::from(1000);

            // TODO: look up decimals from the chain
            // TODO: helper for pretty printing
            info!(
                "min_amount_out USDC: {}",
                BigDecimal::from_u64(min_amount_out_wei.to::<u64>()).unwrap()
                    / BigDecimal::from_f64(1e6).unwrap()
            );

            // TODO: skip sell if min_amount_out is too small

            let allowance = moxie_token
                .allowance(
                    connected_addresses.beneficiary_address,
                    *uniswap_v2_router.address(),
                )
                .call()
                .await?
                ._0;

            if allowance < sell_moxie_wei {
                let approve_hash = moxie_token
                    .approve(*uniswap_v2_router.address(), U256::MAX)
                    .send()
                    .await?
                    .watch()
                    .await?;

                info!("approve hash: {}", approve_hash);

                let approve_transaction = base_provider
                    .get_transaction_by_hash(approve_hash)
                    .await?
                    .wrap_err("approve transaction not found")?;

                info!("approve transaction: {:#?}", approve_transaction);
            }

            // i think its best to get this from the system and not from the chain. but think about this more
            let now = Utc::now().timestamp();

            // deadline = now + 5 minutes
            let deadline = U256::from(now + 5 * 60);

            // TODO: send profits to a different address?
            let to = connected_addresses.beneficiary_address;

            let swap_hash = uniswap_v2_router
                .swapExactTokensForTokens(sell_moxie_wei, min_amount_out_wei, path, to, deadline)
                .send()
                .await?
                .watch()
                .await?;

            info!("swap hash: {}", swap_hash);

            let swap_transaction = base_provider
                .get_transaction_by_hash(swap_hash)
                .await?
                .context("swap transaction not found")?;

            info!("swap transaction: {:#?}", swap_transaction);
        }
        Err(err) => match err.downcast_ref::<NeynarError>() {
            Some(NeynarError::NoButtonTitle(title)) => {
                if title == "Submit & Claim" {
                    info!("already claimed today");
                } else {
                    return Err(err);
                }
            }
            Some(_) => return Err(err),
            None => return Err(err),
        },
    }

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

    debug!("Balances: {:#?}", balances);

    let mut subject_tokens = HashMap::<&str, SubjectTokenData>::new();

    // collect the basic parts of subject tokens into a more useful data structure
    // TODO: we should probably just edit the graphql instead
    for portfolio_tokens_user in balances.iter() {
        for portfolio in portfolio_tokens_user.portfolio.iter() {
            let symbol = portfolio.subject_token.symbol.as_str();

            let subject_token_data =
                subject_tokens
                    .entry(symbol)
                    .or_insert_with(|| SubjectTokenData {
                        symbol,
                        id: &portfolio.subject_token.id,
                        current_price_in_moxie: BigDecimal::from_str(
                            portfolio.subject_token.current_price_in_moxie.as_str(),
                        )
                        .unwrap(),
                        balance_wei: Zero::zero(),
                        display_name: None,
                        avg_daily_earnings: Zero::zero(),
                        earnings_this_week: Zero::zero(),
                        total_supply_wei: BigDecimal::from_str(
                            portfolio.subject_token.total_supply.as_str(),
                        )
                        .unwrap(),
                        user_fans_share_percentage: None,
                    });

            let no_of_tokens = BigInt::from_str(portfolio.no_of_tokens.as_str())
                .context("failed to parse no_of_tokens")?;

            subject_token_data.balance_wei += no_of_tokens;
        }
    }

    let entity_symbols = subject_tokens
        .keys()
        .map(|x| x.to_string())
        .collect::<Vec<_>>();

    let ftas = get_ftas_by_symbol::send_request(
        &https_client,
        get_ftas_by_symbol::Variables {
            entity_symbols: Some(entity_symbols),
        },
    )
    .await?
    .data;

    // TODO: the name on ftas' type is awful
    let ftas = ftas
        .as_ref()
        .context("no token data")?
        .get_ftas
        .ftas
        .as_ref()
        .context("no ftas")?
        .iter()
        .filter_map(|x| x.as_ref())
        .collect::<Vec<_>>();

    for fta in ftas {
        let symbol = fta.entity_symbol.as_deref().unwrap();

        let subject_token_data = subject_tokens.get_mut(symbol).unwrap();

        if let Some(display_name) = &fta.entity_display_name {
            subject_token_data.display_name = Some(display_name.to_string());
        }

        if let Some(avg_daily_earnings) = fta.avg_daily_earnings {
            let avg_daily_earnings = BigDecimal::from_f64(avg_daily_earnings)
                .context("failed to parse avg_daily_earnings")?;
            subject_token_data.avg_daily_earnings = avg_daily_earnings;
        }

        if let Some(earnings_this_week) = fta.earnings_this_week {
            let earnings_this_week = BigDecimal::from_f64(earnings_this_week)
                .context("failed to parse earnings_this_week")?;
            subject_token_data.earnings_this_week = earnings_this_week;
        }

        if let Some(user_fans_share_percentage) = fta.user_fans_share_percentage {
            let user_fans_share_percentage = BigDecimal::from_f64(user_fans_share_percentage)
                .context("failed to parse user_fans_share_percentage")
                .unwrap();

            subject_token_data.user_fans_share_percentage = Some(user_fans_share_percentage);
        } else {
            subject_token_data.user_fans_share_percentage =
                Some(BigDecimal::from_u64(100).unwrap());
        }

        // debug!(
        //     "avg_fan_daily_rewards_per_moxie: {}",
        //     subject_token_data
        //         .avg_fan_daily_rewards_per_moxie()
        //         .to_f32()
        //         .unwrap()
        // );

        // debug!("Fan Token {} stats: {:#?}", symbol, fta);
    }

    // TODO: print the subject tokens in order of `avg_fan_daily_rewards_per_moxie`
    info!("subject_tokens: {:#?}", subject_tokens);

    // TODO: include price impact of buying the fan tokens. do moxie spent * avg_fan_daily_rewards, or do we need to do fan tokens?
    let mut rankings = subject_tokens
        .iter()
        .map(|(symbol, subject_token_data)| {
            (
                *symbol,
                subject_token_data
                    .avg_fan_daily_rewards_per_moxie()
                    .to_f32()
                    .unwrap(),
            )
        })
        .filter(|(_, weekly_rewards_per_moxie)| !weekly_rewards_per_moxie.is_zero())
        .collect::<Vec<_>>();

    // TODO: sorting f32s...
    rankings.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    info!("rankings: {:#?}", rankings);

    // TODO: how many tokens should we pick?
    // TODO: force certain users into the rankings
    // TODO: always buy some percentage of our token
    rankings.truncate((subject_tokens.len() / 4).max(3));

    // TODO: i don't think weights should be linear. raise it to a power?
    let rankings_sum = rankings.iter().map(|(_, x)| x).sum::<f32>();

    // TODO: do we want weights here? might be better to show the raw numbers so we can calculate earnings from our current balance
    let weights = rankings
        .into_iter()
        .map(|(symbol, x)| (symbol, x / rankings_sum))
        .collect::<Vec<_>>();

    info!("top fan tokens by weight: {:#?}", weights);

    let moxie_balance_wei = moxie_token
        .balanceOf(connected_addresses.beneficiary_address)
        .call()
        .await?
        ._0;

    info!("moxie_balance_wei: {}", moxie_balance_wei);

    todo!("buy fan tokens");

    // TODO: sell some moxie for USDC. split the trade between uniswap and aerodrome, or just pick the current best? first pass just use uniswap
    // TODO: mark that we sold some moxie today. that way if we error after this, we don't accidentally sell more on the next run

    // TODO: buy fan tokens according to the weights with the remaining
    // TODO: how do we calculate the price impact? we want to spend 100% of the moxie remaining

    Ok(())
}
