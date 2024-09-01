use std::num::NonZeroUsize;

use serde_json::json;
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
    info!("open_frame: {:#?}", open_frame.frame);

    let open_frame = open_frame
        .click_button("Check rewards", serde_json::Value::Null)
        .await?;
    info!("open_frame check rewards: {:#?}", open_frame.frame);

    if open_frame.frame.image
        != "https://moxie-frames.airstack.xyz/MoxieIntro/airdrop-already-claimed.png"
    {
        todo!("claim the initial airdrop");
    }

    let open_frame = open_frame
        .click_button("View Balance", serde_json::Value::Null)
        .await?;
    info!("open_frame view balance 1: {:#?}", open_frame.frame);

    let left = 84;
    let top = 433;
    let width = 735;
    let height = 142;

    let text = open_frame
        .ocr(left, top, width, height, Some("eng"), None)
        .await?;

    info!("CAPTCHA text: '{}'", text);

    let answer = solve_captcha_math(text)?;

    // let open_frame = open_frame
    //     .click_button(nz::usize!(1), serde_json::Value::Null)
    //     .await?;
    // info!("open_frame view balance 2: {:#?}", open_frame.frame);

    // let open_frame = open_frame
    //     .click_button(nz::usize!(1).unwrap(), serde_json::Value::Null)
    //     .await?;
    // info!("open_frame view balance 3: {:#?}", open_frame.frame);

    // let fetch_captcha_payload = json!({
    //     "action": {
    //         "frames_url": "https://moxie-frames.airstack.xyz/mi",
    //         "post_url": "https://moxie-frames.airstack.xyz/dr/frame",
    //         "button": {
    //             // index 3 = claim button
    //             "index": 3
    //         }
    //     },
    //     "cast_hash": cast_hash,
    //     "signer_uuid": nn_signer_uuid
    // });

    // // TODO: stricter type here
    // let captcha_response: serde_json::Value = client
    //     .post("https://api.neynar.com/v2/farcaster/frame/action")
    //     .header("api_key", nn_api_key)
    //     .json(&fetch_captcha_payload)
    //     .send()
    //     .await?
    //     .error_for_status()?
    //     .json()
    //     .await?;

    // // TODO: option to send a message with the captcha image so the user can solve it themselves
    // info!("captcha_response: {:#?}", captcha_response);
    // // TODO: solve a captcha...? wtf guys...? i guess that scraps this

    // // TODO: turn the jpeg base64 into something that tesseract can read
    // let image_path = "30_plus_5.jpg";

    // let answer = solve_captcha(image_path, left, top, width, height)?;

    todo!();

    // TODO: get frame url and post
    // let submit_captcha_payload = json!({
    //     "action": {
    //         "frames_url": "https://moxie-frames.airstack.xyz/dr/frame?r=&c=&f=2",
    //         "post_url": "https://moxie-frames.airstack.xyz/dr/frame?r=&c=elpxYfW644Hyeboc9np9H",
    //         "button": {
    //             // index 2 = Submit & Claim
    //             "index": 2
    //         },
    //         "input": answer.to_string()
    //     },
    //     "cast_hash": cast_hash,
    //     "signer_uuid": nn_signer_uuid
    // });

    // let response = client
    //     .post("https://api.neynar.com/v2/farcaster/frame/action")
    //     .header("api_key", nn_api_key)
    //     .json(&submit_captcha_payload)
    //     .send()
    //     .await?;

    // let response = response.json::<Response>().await?;

    // TODO: sleep 10 seconds?

    // TODO: load the next page of the frame

    Ok(())
}

pub async fn buy_fan_tokens() {
    // TODO: parse <https://moxie-frames.airstack.xyz/stim?t=fid_206>

    todo!();
}
