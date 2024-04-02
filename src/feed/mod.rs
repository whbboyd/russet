pub mod atom;
pub mod model;

use crate::Result;
use model::Feed;
use reqwest::Url;

#[allow(async_fn_in_trait)]
pub trait RussetFeedReader {
	async fn load_feed(&self, url: &Url) -> Result<Feed>;
}
