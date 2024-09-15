//! Use Neynar APIs to crawl a frame.

use alloy::primitives::Address;
use anyhow::Context;
use base64::prelude::{Engine, BASE64_STANDARD};
use petgraph::{graph::NodeIndex, Graph};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, num::NonZeroUsize, sync::Arc};
use terrors::OneOf;
use tesseract::Tesseract;
use tracing::{debug, info, warn};

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
    pub eth_addresses: Vec<Address>,
    /// TODO: what type on this? its not an ETH address. its longer (and base58)
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
#[serde(rename_all = "camelCase")]
pub struct Html {
    pub charset: String,
    pub favicon: Option<String>,
    pub og_description: Option<String>,
    pub og_image: Vec<OgImage>,
    pub og_locale: Option<String>,
    pub og_title: Option<String>,
    pub twitter_card: Option<String>,
    pub twitter_description: Option<String>,
    pub twitter_image: Option<Vec<OgImage>>,
    pub twitter_title: Option<String>,
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

#[derive(Clone, Deserialize, Debug, Serialize)]
pub struct Button {
    pub action_type: String,
    /// TODO: this should be nonzero!
    pub index: usize,
    pub target: Option<String>,
    pub title: Option<String>,
    pub post_url: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Clone, Deserialize, Debug, Serialize)]
pub struct Input {
    pub text: Option<String>,
    // #[serde(flatten)]
    // pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Clone, Deserialize, Debug, Serialize)]
pub struct State {
    pub serialized: Option<String>,
    // #[serde(flatten)]
    // pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Clone, Deserialize, Debug)]
pub struct Frame {
    pub buttons: Vec<Button>,
    pub frames_url: Option<String>,
    pub image: Option<String>,
    pub input: Option<Input>,
    pub post_url: Option<String>,
    pub state: Option<State>,
    pub title: Option<String>,
    pub version: Option<String>,
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
    address: Address,
    neynar_client: reqwest::Client,
    neynar_signer_uuid: String,
}

pub struct OpenFrame<'a> {
    cast: Arc<Cast>,
    crawler: &'a FrameCrawler,
    pub frame: Frame,
}

#[derive(Debug, serde::Deserialize)]
struct NeynarFrameActionResponse {
    version: Option<String>,
    title: Option<String>,
    image: Option<String>,
    buttons: Option<Vec<Button>>,
    /// TODO: what type?
    input: Option<Input>,
    /// TODO: what type?
    state: Option<State>,
    frames_url: Option<String>,
    post_url: Option<String>,
    image_aspect_ratio: Option<String>,
    /// an error message
    /// TODO: if this is set, throw an error
    message: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl TryFrom<NeynarFrameActionResponse> for Frame {
    type Error = anyhow::Error;

    fn try_from(response: NeynarFrameActionResponse) -> anyhow::Result<Self> {
        debug!("NeynarFrameActionResponse: {:#?}", response);

        let frames_url = response.frames_url;
        let image = response.image;
        let input = response.input;
        let post_url = response.post_url;
        let state = response.state;
        let title = response.title;
        let version = response.version;
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
    pub async fn new(
        address: Address,
        neynar_api_key: String,
        neynar_signer_uuid: String,
    ) -> anyhow::Result<Self> {
        let neynar_client = neynar_client(&neynar_api_key)?;

        let x = Self {
            address,
            neynar_client,
            neynar_signer_uuid,
        };

        Ok(x)
    }

    /// TODO: return a Cast, not a Value
    pub async fn get_cast_by_hash(
        &self,
        cast_hash: &str,
    ) -> Result<
        Cast,
        OneOf<(
            reqwest::Error,
            serde_path_to_error::Error<serde_json::Error>,
        )>,
    > {
        let j = self
            .neynar_client
            .get("https://api.neynar.com/v2/farcaster/cast")
            .query(&[("identifier", cast_hash), ("type", "hash")])
            .send()
            .await
            .map_err(OneOf::new)?
            .error_for_status()
            .map_err(OneOf::new)?
            .text()
            .await
            .map_err(OneOf::new)?;

        let jd = &mut serde_json::Deserializer::from_str(&j);

        let cast_result: Result<CastResponse, _> = serde_path_to_error::deserialize(jd);

        let cast_response = cast_result.map_err(OneOf::new)?;

        Ok(cast_response.cast)
    }

    /// TODO: terrors instead of anyhow
    pub async fn open_frame(
        &self,
        cast_hash: &str,
        frame_index: usize,
    ) -> anyhow::Result<OpenFrame> {
        let cast = self
            .get_cast_by_hash(cast_hash)
            .await
            .map_err(|e| anyhow::anyhow!("{:#?}", e))?;

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

        info!("opened frame: {:#?}", open_frame.frame);

        Ok(open_frame)
    }

    pub async fn crawl_frame<'a>(
        &'a self,
        cast_hash: &str,
        frame_index: usize,
    ) -> anyhow::Result<(Arc<Cast>, Graph<Frame, String>)> {
        let mut graph = Graph::new();

        let first = self.open_frame(cast_hash, frame_index).await?;

        let cast = first.cast.clone();

        // TODO: add the cast to the graph. i think we need an enum then

        first.crawl(&mut graph).await?;

        Ok((cast, graph))
    }
}

impl OpenFrame<'_> {
    pub async fn crawl(&self, graph: &mut Graph<Frame, String>) -> anyhow::Result<NodeIndex> {
        // first we add self to the graph
        let a = graph.add_node(self.frame.clone());

        if self.frame.input.is_some() {
            warn!(?self.frame.input, "The frame might require input!");
        }

        // then we iterate over the buttons and call crawl on them
        // TODO: spawn this so it can be done in parallel. need a lock on the map and graph then though
        for next_button in self.frame.buttons.iter() {
            let next_frame = self
                .click_button_index(NonZeroUsize::new(next_button.index).unwrap(), None)
                .await?;

            // box so that recursion works
            let b = Box::pin(next_frame.crawl(graph)).await?;

            let edge_label = next_button
                .title
                .clone()
                .unwrap_or_else(|| format!("Button #{}", next_button.index));

            graph.add_edge(a, b, edge_label);
        }

        Ok(a)
    }

    /// [See](https://docs.neynar.com/reference/post-frame-action)
    /// TODO: i think this might need to return an enum. Sometimes things are transactions are external links
    pub async fn click_button_index(
        &self,
        button_index: NonZeroUsize,
        input_text: Option<&str>,
    ) -> anyhow::Result<OpenFrame> {
        /// TODO: version,title,image
        /// TODO: better types for transaction
        #[derive(Debug, Serialize)]
        struct ActionObject<'a> {
            button: Button,
            input: Option<Input>,
            state: Option<State>,
            #[serde(skip_serializing_if = "serde_json::Value::is_null")]
            transaction: serde_json::Value,
            address: Option<Address>,
            frames_url: &'a str,
            post_url: &'a str,
        }

        #[derive(Debug, Serialize)]
        struct FramePayload<'a> {
            signer_uuid: &'a str,
            cast_hash: &'a str,
            action: ActionObject<'a>,
        }

        let button = self
            .frame
            .buttons
            .iter()
            .find(|x| x.index == button_index.get())
            .context("no button with that index")?;

        // TODO: support other types of actions
        anyhow::ensure!(button.action_type == "post", "button is not a post");

        let input = input_text.map(|input_text| Input {
            text: Some(input_text.to_string()),
            // extra: Default::default(),
        });

        // TODO: not sure about transaction
        let payload = FramePayload {
            action: ActionObject {
                post_url: self.frame.post_url.as_deref().unwrap(),
                frames_url: self.frame.frames_url.as_deref().unwrap(),
                button: Button {
                    title: button.title.clone(),
                    index: button_index.get(),
                    action_type: button.action_type.clone(),
                    post_url: None,
                    target: button.target.clone(),
                    extra: Default::default(),
                },
                input,
                state: self.frame.state.clone(),
                transaction: serde_json::Value::Null,
                address: Some(self.crawler.address),
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

        info!("opened with click: {:#?}", open_frame.frame);

        Ok(open_frame)
    }

    pub async fn click_button(
        &self,
        button_title: &str,
        input_text: Option<&str>,
    ) -> anyhow::Result<OpenFrame> {
        let button = self
            .frame
            .buttons
            .iter()
            .find(|button| button.title.as_deref() == Some(button_title))
            .context(format!("no button with the title '{}'", button_title))?;

        let button_index = button.index;

        // TODO: short, random sleep here?

        let open_frame = self
            .click_button_index(NonZeroUsize::new(button_index).unwrap(), input_text)
            .await?;

        Ok(open_frame)
    }

    pub async fn ocr(
        &self,
        left: i32,
        top: i32,
        width: i32,
        height: i32,
        language: Option<&str>,
        whitelist: Option<&str>,
    ) -> anyhow::Result<String> {
        let mut tess = Tesseract::new(None, language)?;

        let image = self.frame.image.as_deref().context("no image")?;

        tess = if image.starts_with("http") {
            let image_data = reqwest::get(image)
                .await?
                .error_for_status()?
                .bytes()
                .await?;

            // TODO: display the image

            // TODO: save the image to a temporary file and then let tess open it
            tess.set_image_from_mem(image_data.as_ref())?
        } else if image.starts_with("data:") {
            let image_data = image
                .split_once(',')
                .map(|x| x.1)
                .context("no image data")?;

            // TODO: display the image

            let image_data = BASE64_STANDARD.decode(image_data)?;

            // TODO: i don't think this is right. i think we need an "image" crate here to decode the image. maybe we should just save to a file and let tess open it
            tess.set_image_from_mem(&image_data)?
        } else {
            anyhow::bail!("image is not a URL or data");
        };

        if let Some(whitelist) = whitelist {
            tess = tess.set_variable("tessedit_char_whitelist", whitelist)?;
        }

        tess = tess.set_rectangle(left, top, width, height);

        tess = tess.recognize()?;

        let text = tess.get_text()?;

        Ok(text)
    }
}

mod test {
    // TODO: why are these showing as unused?
    use super::*;
    use alloy::primitives::address;
    use tracing::Level;

    #[tokio::test]
    async fn test_crawl_yoink() {
        dotenvy::dotenv().expect(".env file is needed for credentials");

        // TODO: log init should be in a test helper. turn on DEBUG
        tracing_subscriber::fmt()
            .pretty()
            .with_max_level(Level::DEBUG)
            .init();

        let cast_hash = "0x9f748161eca76edfa6363140b4ef9317386f8e3b";

        let neynar_signer_uuid = std::env::var("NEYNAR_SIGNER_UUID").unwrap();
        let neynar_api_key = std::env::var("NEYNAR_API_KEY").unwrap();

        // TODO: wait. who is this address? and why does my signer work for it? address should come from .env
        // TODO: this is rish's address but my signer. this should fail!
        let address = address!("e1fac64cebe0855d984ac7c3feb8b7612c6b4176");

        let frame_crawler = FrameCrawler::new(address, neynar_api_key, neynar_signer_uuid)
            .await
            .unwrap();

        let page_0 = frame_crawler.open_frame(cast_hash, 0).await.unwrap();

        assert_eq!(page_0.frame.title.as_deref(), Some("Yoink"));

        let page_1 = page_0.click_button("🚩 Start", None).await.unwrap();

        assert_eq!(page_1.frame.title.as_deref(), Some("Yoink!"));
        // // TODO: assert more things

        // // "Yoink!" button only works if address matches. you'll get unauthorized here otherwise.
        // // TODO: the response code is 200 though. need better errors. need to have neynar send the http code smarter
        // let page_2 = page_1.click_button("Yoink!", None).await.unwrap();
    }
}
