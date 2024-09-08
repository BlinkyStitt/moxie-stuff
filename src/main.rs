use std::{collections::HashMap, str::FromStr};

use anyhow::Context;
use bigdecimal::{BigDecimal, ToPrimitive};
use moxie_stuff::{
    airstack_connected_addresses, claim_everyday_rewards_with_neynar, get_ftas_by_symbol,
    https_client, portfolio_tokens, FrameCrawler,
};
use num_bigint::BigInt;
use num_traits::{FromPrimitive, Zero};
use tracing::info;

#[derive(serde::Deserialize)]
struct Config {
    airstack_api_key: Option<String>,
    farcaster_id: i64,
    neynar_api_key: String,
    neynar_signer_uuid: String,
}

struct SubjectTokenData<'a> {
    symbol: &'a str,
    id: &'a str,
    display_name: Option<String>,
    current_price_in_moxie: BigDecimal,
    total_supply_wei: BigDecimal,
    balance_wei: BigDecimal,
    avg_daily_earnings: BigDecimal,
    earnings_this_week: BigDecimal,
}

impl SubjectTokenData<'_> {
    fn apr(&self) -> BigDecimal {
        self.avg_daily_rewards_per_moxie()
            * BigDecimal::from_u64(365).unwrap()
            * BigDecimal::from_u64(100).unwrap()
    }

    fn avg_daily_rewards_per_moxie(&self) -> BigDecimal {
        // TODO: &self.avg_daily_earnings / &self.tvl? or tvl/avg?
        &self.avg_daily_rewards_per_fan_token() / &self.current_price_in_moxie
    }

    /// TODO: this feels wrong. i think we want to use TVL here instead?
    /// TODO: SEE: <https://mirror.xyz/siddxa.eth/wgGqLMmYzKhZtBVBPPNPyJaxNF_GIJAo2MwlFMV4Hvo>
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
            .finish()
    }
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
    // let airstack_claims_client = airstack_claims_client(&config.airstack_api_key)?;

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
    // TODO: better error types so that we can skip if we get an "already claimed"/"no claim button" error
    // claim_everyday_rewards_with_neynar(&frame_crawler).await?;

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

    // TODO: no need to zip. the fta has the symbol in it somewhere
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

        // TODO: also include the fan token amount in the calculation. some tokens give 20%. others give 100%.
        info!(
            "weekly_rewards_per_moxie: {}",
            subject_token_data
                .avg_daily_rewards_per_moxie()
                .to_f64()
                .unwrap()
        );

        info!("Fan Token {} stats: {:#?}", symbol, fta);
    }

    info!("subject_tokens: {:#?}", subject_tokens);

    let mut rankings = subject_tokens
        .iter()
        .map(|(symbol, subject_token_data)| {
            (symbol, subject_token_data.avg_daily_rewards_per_moxie())
        })
        .filter(|(_, weekly_rewards_per_moxie)| !weekly_rewards_per_moxie.is_zero())
        .collect::<Vec<_>>();

    rankings.sort_by(|a, b| b.1.cmp(&a.1));

    // TODO: how many tokens should we pick?
    // TODO: force certain users into the rankings
    // TODO: always buy some percentage of our token
    rankings.truncate((subject_tokens.len() / 4).max(2));

    let rankings_sum = rankings.iter().map(|(_, x)| x).sum::<BigDecimal>();

    let weights = rankings
        .into_iter()
        .map(|(symbol, x)| (symbol, (x / &rankings_sum).to_f32().unwrap()))
        .collect::<Vec<_>>();

    info!("top fan tokens: {:#?}", weights);

    // TODO: figure out how much moxie we have to spend. keep a slush fund of some USD value?

    // TODO: sell some moxie for

    // TODO: buy fan tokens

    Ok(())
}
