//! Use Neynar APIs to crawl a frame.

use anyhow::Context;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{collections::HashMap, num::NonZeroUsize, sync::Arc};
use tracing::info;

use crate::neynar_client;

#[derive(Deserialize, Debug)]
pub struct Profile {
    pub text: String,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Deserialize, Debug)]
pub struct Bio {
    pub bio: Profile,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Deserialize, Debug)]
pub struct VerifiedAddresses {
    pub eth_addresses: Vec<String>,
    pub sol_addresses: Vec<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Deserialize, Debug)]
pub struct Author {
    pub active_status: String,
    pub custody_address: String,
    pub display_name: String,
    pub fid: u64,
    pub follower_count: u64,
    pub following_count: u64,
    pub object: String,
    pub pfp_url: String,
    pub power_badge: bool,
    pub profile: Bio,
    pub username: String,
    pub verifications: Vec<String>,
    pub verified_addresses: VerifiedAddresses,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Deserialize, Debug)]
pub struct Channel {
    pub id: String,
    pub image_url: String,
    pub name: String,
    pub object: String,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Deserialize, Debug)]
pub struct OgImage {
    pub url: String,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Deserialize, Debug)]
pub struct Html {
    pub charset: String,
    pub favicon: Option<String>,
    pub ogDescription: Option<String>,
    pub ogImage: Vec<OgImage>,
    pub ogLocale: Option<String>,
    pub ogTitle: Option<String>,
    pub twitterCard: Option<String>,
    pub twitterDescription: Option<String>,
    pub twitterImage: Option<Vec<OgImage>>,
    pub twitterTitle: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Deserialize, Debug)]
pub struct Metadata {
    pub _status: String,
    pub content_length: Option<u64>,
    pub content_type: String,
    pub html: Html,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Deserialize, Debug)]
pub struct Embed {
    pub metadata: Metadata,
    pub url: String,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Clone, Deserialize, Debug)]
pub struct Button {
    pub action_type: String,
    pub index: u64,
    pub target: Option<String>,
    pub title: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Clone, Deserialize, Debug)]
pub struct Frame {
    pub buttons: Vec<Button>,
    pub frames_url: String,
    pub image: String,
    pub input: serde_json::Value, // what type?
    pub post_url: String,
    pub state: serde_json::Value, // what type?
    pub title: Option<String>,
    pub version: String,
    pub image_aspect_ratio: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Deserialize, Debug)]
pub struct Reaction {
    pub fid: u64,
    pub fname: String,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Deserialize, Debug)]
pub struct Reactions {
    pub likes: Vec<Reaction>,
    pub likes_count: u64,
    pub recasts: Vec<Reaction>,
    pub recasts_count: u64,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Deserialize, Debug)]
pub struct Replies {
    pub count: u64,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Deserialize, Debug)]
pub struct Cast {
    pub author: Author,
    pub channel: Channel,
    pub embeds: Vec<Embed>,
    pub frames: Vec<Frame>,
    pub hash: String,
    pub mentioned_profiles: Vec<serde_json::Value>, // Placeholder for the unknown structure
    pub object: String,
    pub parent_author: serde_json::Value, // Placeholder for the unknown structure
    pub parent_hash: Option<String>,
    pub parent_url: String,
    pub reactions: Reactions,
    pub replies: Replies,
    pub root_parent_url: String,
    pub text: String,
    pub thread_hash: String,
    pub timestamp: String,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Deserialize, Debug)]
pub struct CastResponse {
    pub cast: Cast,
}

#[derive(Deserialize, Debug)]
pub struct Root {
    pub cast: Cast,
}

pub struct FrameCrawler {
    neynar_client: reqwest::Client,
    neynar_signer_uuid: String,
}

pub struct OpenFrame<'a> {
    cast: Arc<Cast>,
    crawler: &'a FrameCrawler,
    pub frame: Frame,
}

#[allow(dead_code)]
#[derive(Debug, serde::Deserialize)]
struct NeynarFrameActionResponse {
    version: Option<String>,
    title: Option<String>,
    image: Option<String>,
    buttons: Option<Vec<Button>>,
    /// TODO: what type?
    input: serde_json::Value,
    /// TODO: what type?
    state: serde_json::Value,
    frames_url: String,
    post_url: String,
    image_aspect_ratio: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl TryFrom<NeynarFrameActionResponse> for Frame {
    type Error = anyhow::Error;

    fn try_from(response: NeynarFrameActionResponse) -> anyhow::Result<Self> {
        let frames_url = response.frames_url;
        let image = response.image.context("image")?;
        let input = response.input;
        let post_url = response.post_url;
        let state = response.state;
        let title = response.title;
        let version = response.version.context("version")?;
        let buttons = response.buttons.context("buttons")?;
        let image_aspect_ratio = response.image_aspect_ratio;
        let extra = response.extra;

        let x = Self {
            buttons,
            frames_url,
            image,
            input,
            post_url,
            state,
            title,
            version,
            image_aspect_ratio,
            extra,
        };

        Ok(x)
    }
}

impl FrameCrawler {
    pub async fn new(neynar_api_key: String, neynar_signer_uuid: String) -> anyhow::Result<Self> {
        let neynar_client = neynar_client(&neynar_api_key)?;

        let x = Self {
            neynar_client,
            neynar_signer_uuid,
        };

        Ok(x)
    }

    /// TODO: return a Cast, not a Value
    pub async fn get_cast_by_hash(&self, cast_hash: &str) -> reqwest::Result<Cast> {
        let x: CastResponse = self
            .neynar_client
            .get("https://api.neynar.com/v2/farcaster/cast")
            .query(&[("identifier", cast_hash), ("type", "hash")])
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        Ok(x.cast)
    }

    /// TODO: return a Frame object that's wrapped with this cast hash and crawler
    pub async fn open_frame(
        &self,
        cast_hash: &str,
        frame_index: usize,
    ) -> anyhow::Result<OpenFrame> {
        let cast = self.get_cast_by_hash(cast_hash).await?;

        let frame = cast
            .frames
            .get(frame_index)
            .context("no frame with that index")?
            .clone();

        let open_frame = OpenFrame {
            frame,
            cast: Arc::new(cast),
            crawler: self,
        };

        Ok(open_frame)
    }
}

impl OpenFrame<'_> {
    /// TODO: type to keep the frame and cast_hash together
    /// TODO: i think this might need to return an enum. Sometimes things are transactions are external links
    /// TODO: helper that takes the title of the button instead of the index
    pub async fn click_button(
        &self,
        button_index: NonZeroUsize,
        input: serde_json::Value,
    ) -> anyhow::Result<OpenFrame> {
        #[derive(Debug, Serialize)]
        struct ButtonAction {
            index: usize,
            #[serde(skip_serializing_if = "serde_json::Value::is_null")]
            input: serde_json::Value,
            #[serde(skip_serializing_if = "serde_json::Value::is_null")]
            state: serde_json::Value,
        }

        #[derive(Debug, Serialize)]
        struct FrameAction<'a> {
            frames_url: &'a str,
            post_url: &'a str,
            button: ButtonAction,
        }

        #[derive(Debug, Serialize)]
        struct FramePayload<'a> {
            action: FrameAction<'a>,
            cast_hash: &'a str,
            signer_uuid: &'a str,
        }

        let button = self
            .frame
            .buttons
            .get(button_index.get() - 1)
            .context("no button with that index")?;

        // TODO: support other types of actions
        anyhow::ensure!(button.action_type == "post", "button is not a post");

        let payload = FramePayload {
            action: FrameAction {
                post_url: button
                    .target
                    .as_deref()
                    .unwrap_or(self.frame.post_url.as_str()),
                frames_url: &self.frame.frames_url,
                button: ButtonAction {
                    index: button_index.get(),
                    input,
                    state: self.frame.state.clone(),
                },
            },
            cast_hash: &self.cast.hash,
            signer_uuid: &self.crawler.neynar_signer_uuid,
        };

        let response = self
            .crawler
            .neynar_client
            .post("https://api.neynar.com/v2/farcaster/frame/action")
            .json(&payload)
            .send()
            .await?;

        let response = response.json::<NeynarFrameActionResponse>().await?;

        let frame = Frame::try_from(response)?;

        let open_frame = OpenFrame {
            frame,
            cast: self.cast.clone(),
            crawler: self.crawler,
        };

        Ok(open_frame)
    }
}

mod test {
    use super::*;

    #[tokio::test]
    async fn test_crawl_yoink() {
        dotenvy::dotenv().unwrap();

        let cast_hash = "0x9f748161eca76edfa6363140b4ef9317386f8e3b";

        let nn_signer_uuid = std::env::var("NEYNAR_SIGNER_UUID").unwrap();
        let nn_api_key = std::env::var("NEYNAR_API_KEY").unwrap();

        let frame_crawler = FrameCrawler::new(nn_api_key, nn_signer_uuid).await.unwrap();

        let page_0 = frame_crawler.open_frame(cast_hash, 0).await.unwrap();

        assert_eq!(page_0.frame.title, Some("Yoink".to_string()));

        let page_1 = page_0
            .click_button(2.try_into().unwrap(), serde_json::Value::Null)
            .await
            .unwrap();

        assert_eq!(page_1.frame.title, Some("Yoink!".to_string()));

        // TODO: more to assert?
    }
}
