pub mod atom;
pub mod model;

use crate::Result;
use model::Feed;

pub trait RussetFeedReader {
	fn read_feed(&self, bytes: &[u8]) -> Result<Feed>;
}
