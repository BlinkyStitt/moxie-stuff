use serde_json::json;
use tracing::info;
use url::Url;

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

pub async fn claim_everyday_rewards_with_neynar(
    client: &reqwest::Client,
    nn_signer_uuid: &str,
    nn_api_key: &str,
) -> reqwest::Result<()> {
    let cast_hash = "0x97906c211fa5f48d4377ddc1e2b5547e428b4c8e";

    let fetch_captcha_payload = json!({
        "action": {
            "frames_url": "https://moxie-frames.airstack.xyz/mi",
            "post_url": "https://moxie-frames.airstack.xyz/dr/frame",
            "button": {
                // index 3 = claim button
                "index": 3
            }
        },
        "cast_hash": cast_hash,
        "signer_uuid": nn_signer_uuid
    });

    let captcha_response = client
        .post("https://api.neynar.com/v2/farcaster/frame/action")
        .header("api_key", nn_api_key)
        .json(&fetch_captcha_payload)
        .send()
        .await?;

    info!("captcha_response: {:#?}", captcha_response);
    // TODO: solve a captcha...? wtf guys...? i guess that scraps this

    todo!();

    // TODO: get frame url and post
    let submit_captcha_payload = json!({
        "action": {
            "frames_url": "https://moxie-frames.airstack.xyz/dr/frame?r=&c=&f=2",
            "post_url": "https://moxie-frames.airstack.xyz/dr/frame?r=&c=elpxYfW644Hyeboc9np9H",
            "button": {
                // index 2 = Submit & Claim
                "index": 2
            }
        },
        "cast_hash": cast_hash,
        "signer_uuid": nn_signer_uuid
    });

    let response = client
        .post("https://api.neynar.com/v2/farcaster/frame/action")
        .header("api_key", nn_api_key)
        .json(&submit_captcha_payload)
        .send()
        .await?;

    let response = response.json::<Response>().await?;

    // TODO: sleep 10 seconds?

    // TODO: load the next page of the frame

    Ok(())
}

pub async fn buy_fan_tokens() {
    // TODO: parse <https://moxie-frames.airstack.xyz/stim?t=fid_206>

    todo!();
}
