pub mod atom;
pub mod model;
pub mod rss;

use crate::Result;
use model::Feed;

pub trait RussetFeedReader {
	fn read_feed(&self, bytes: &[u8]) -> Result<Feed>;
}
