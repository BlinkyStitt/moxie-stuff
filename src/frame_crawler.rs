//! Use Neynar APIs to crawl a frame.

use anyhow::Context;
use base64::prelude::{Engine, BASE64_STANDARD};
use petgraph::{graph::NodeIndex, Graph};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    num::{NonZero, NonZeroUsize},
    sync::Arc,
};
use tesseract::Tesseract;

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
    /// TODO: this should be nonzero!
    pub index: usize,
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
    address: String,
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
    pub async fn new(
        address: String,
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

    pub async fn crawl_frame<'a>(
        &'a self,
        cast_hash: &str,
        frame_index: usize,
    ) -> anyhow::Result<Graph<String, String>> {
        let mut frames = HashMap::<NodeIndex, Frame>::new();

        let mut graph = Graph::new();

        let first = self.open_frame(cast_hash, frame_index).await?;

        let first_frame = first.frame;

        // TODO: what should the weight be? image? title? some combination? a render of the frame with buttons?

        // TODO: recurse through the frames. pass &mut frames and &mut graph so it can add itself

        Ok(graph)
    }
}

impl OpenFrame<'_> {
    pub async fn crawl(
        &self,
        frames: &mut HashMap<NodeIndex, Frame>,
        graph: &mut Graph<String, String>,
    ) -> anyhow::Result<()> {
        // first we add self to the graph
        let x = graph.add_node(self.frame.image.clone());

        frames.insert(x, self.frame.clone());

        // then we iterate over the buttons and call crawl on them
        // TODO: spawn this so it can be done in parallel. need a lock on the map and graph then though
        for next_button in self.frame.buttons.iter() {
            let next_frame = self
                .click_button_index(NonZeroUsize::new(next_button.index).unwrap(), None)
                .await?;

            // box so that recursion works
            Box::pin(next_frame.crawl(frames, graph)).await?;
        }

        Ok(())
    }

    /// [See](https://docs.neynar.com/reference/post-frame-action)
    /// TODO: i think this might need to return an enum. Sometimes things are transactions are external links
    pub async fn click_button_index(
        &self,
        button_index: NonZeroUsize,
        input_text: Option<&str>,
    ) -> anyhow::Result<OpenFrame> {
        /// TODO: title,target,post_url of the button is part of the protobuf, but i don't think we need it
        #[derive(Debug, Serialize)]
        struct ButtonObject<'a> {
            title: Option<&'a str>,
            index: usize,
            action_type: &'a str,
            target: Option<&'a str>,
            post_url: Option<&'a str>,
        }

        #[derive(Debug, Serialize)]
        struct InputObject<'a> {
            text: &'a str,
        }

        /// TODO: version,title,image
        /// TODO: better types for input,state,transaction
        #[derive(Debug, Serialize)]
        struct ActionObject<'a> {
            button: ButtonObject<'a>,
            input: Option<InputObject<'a>>,
            #[serde(skip_serializing_if = "serde_json::Value::is_null")]
            state: serde_json::Value,
            #[serde(skip_serializing_if = "serde_json::Value::is_null")]
            transaction: serde_json::Value,
            // #[serde(skip_serializing_if = "serde_json::Value::is_null")]
            address: Option<String>,
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

        let input = if let Some(input_text) = input_text {
            Some(InputObject { text: input_text })
        } else {
            None
        };

        // TODO: not sure about input or state or post_url lol
        let payload = FramePayload {
            action: ActionObject {
                /*
                post_url: button
                    .target
                    .as_deref()
                    .unwrap_or(self.frame.post_url.as_str()),
                */
                post_url: self.frame.post_url.as_str(),
                frames_url: &self.frame.frames_url,
                button: ButtonObject {
                    title: None,
                    index: button_index.get(),
                    action_type: &button.action_type,
                    post_url: None,
                    target: button.target.as_deref(),
                },
                input,
                state: self.frame.state.clone(),
                transaction: serde_json::Value::Null,
                address: None,
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
            .context("no button with that title")?;

        let button_index = button.index;

        self.click_button_index(NonZeroUsize::new(button_index).unwrap(), input_text)
            .await
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

        tess = if self.frame.image.starts_with("http") {
            let image_data = reqwest::get(self.frame.image.clone())
                .await?
                .error_for_status()?
                .bytes()
                .await?;

            // TODO: display the image

            // TODO: save the image to a temporary file and then let tess open it
            tess.set_image_from_mem(image_data.as_ref())?
        } else if self.frame.image.starts_with("data:") {
            let image_data = self
                .frame
                .image
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
    use super::*;

    #[tokio::test]
    async fn test_crawl_yoink() {
        dotenvy::dotenv().unwrap();

        let cast_hash = "0x9f748161eca76edfa6363140b4ef9317386f8e3b";

        let neynar_signer_uuid = std::env::var("NEYNAR_SIGNER_UUID").unwrap();
        let neynar_api_key = std::env::var("NEYNAR_API_KEY").unwrap();
        let address = "0x97906c211fa5f48d4377ddc1e2b5547e428b4c8e".to_string();

        let frame_crawler = FrameCrawler::new(address, neynar_api_key, neynar_signer_uuid)
            .await
            .unwrap();

        let page_0 = frame_crawler.open_frame(cast_hash, 0).await.unwrap();

        assert_eq!(page_0.frame.title.as_deref(), Some("Yoink"));

        let page_1 = page_0.click_button("🚩 Start", None).await.unwrap();

        assert_eq!(page_1.frame.title.as_deref(), Some("Yoink!"));
        // TODO: assert more things
    }
}
