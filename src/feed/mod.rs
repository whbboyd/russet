pub mod atom;
pub mod model;

use crate::Result;
use model::Feed;
use reqwest::Url;

pub trait RussetFeedReader {
	fn load_feed(&self, url: &Url) -> Result<Feed>;
}
