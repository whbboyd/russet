pub mod model;
pub mod sql;

use crate::Result;
use model::{ Entry, Feed };
use reqwest::Url;
use ulid::Ulid;

pub trait RussetPersistanceLayer {

	fn add_feed(&mut self, feed: &Feed) -> Result<()>;

	fn get_feeds(&self) -> impl IntoIterator<Item = Result<Feed>>;

	fn get_feed(&self, id: &Ulid) -> Result<Feed>;

	fn get_feed_by_url(&self, url: &Url) -> Result<Feed>;

	fn get_entry(&self, id: &Ulid) -> Result<Entry>;

	fn get_entries_for_feed(&self, feed_id: &Ulid) -> impl IntoIterator<Item = Result<Entry>>;

	fn get_and_increment_fetch_index(&mut self) -> Result<u32>;
}
