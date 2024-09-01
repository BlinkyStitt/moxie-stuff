use std::{num::NonZeroUsize, time::Duration};

use serde_json::json;
use tokio::time::sleep;
use tracing::info;
use url::Url;

use crate::{frame_crawler::FrameCrawler, solve_captcha, solve_captcha_math};

// TODO: FrameBrowser struct that has "set input" and "click button" methods. take a cast or a frame url as a starting point

#[allow(dead_code)]
#[derive(Debug, serde::Deserialize)]
struct Response {
    version: Option<String>,
    title: Option<String>,
    image: Url,
    // buttons: Vec<serde_json::Value>,
    // input: serde_json::Value,
    // state: serde_json::Value,
    // frames_url: String,
}

pub async fn claim_everyday_rewards_with_neynar(crawler: &FrameCrawler) -> anyhow::Result<()> {
    let cast_hash = "0x97906c211fa5f48d4377ddc1e2b5547e428b4c8e";

    let open_frame = crawler.open_frame(cast_hash, 0).await?;

    let open_frame = open_frame.click_button("Check rewards", None).await?;

    if open_frame.frame.image
        != "https://moxie-frames.airstack.xyz/MoxieIntro/airdrop-already-claimed.png"
    {
        anyhow::bail!("claim the initial airdrop");
    }

    let open_frame = open_frame.click_button("View Balance", None).await?;

    let open_frame = open_frame.click_button("Rewards", None).await?;

    // TODO: check to see if its a message about us already having claimed today. return Ok(()) instead of "button not found"
    let open_frame = open_frame.click_button("Claim", None).await?;

    let left = 84;
    let top = 433;
    let width = 735;
    let height = 142;

    let text = open_frame
        .ocr(left, top, width, height, Some("eng"), None)
        .await?;

    info!("CAPTCHA text: '{}'", text);

    let answer = solve_captcha_math(text)?;

    info!("CAPTCHA answer: '{}'", answer);

    let open_frame = open_frame
        .click_button("Submit & Claim", Some(&format!("{}", answer)))
        .await?;

    sleep(Duration::from_secs(10)).await;

    let open_frame = open_frame
        .click_button("Check status", Some(&format!("{}", answer)))
        .await?;

    // TODO: check the claim status and error or loop

    Ok(())
}

pub async fn buy_fan_tokens() {
    // TODO: parse <https://moxie-frames.airstack.xyz/stim?t=fid_206>

    todo!();
}
