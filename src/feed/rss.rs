use crate::feed::model::Entry;
use crate::feed::model::Feed;
use crate::feed::RussetFeedReader;
use crate::Result;
use reqwest::Url;
use rss::Channel;
use std::time::SystemTime;

pub struct RssFeedReader { }
impl RssFeedReader {
	pub fn new() -> RssFeedReader {
		RssFeedReader{ }
	}
}
impl RussetFeedReader for RssFeedReader {

	fn read_feed(&self, bytes: &[u8]) -> Result<Feed> {
		let rss = Channel::read_from(bytes)?;
		let title = rss.title;
		let entries = rss.items.into_iter().map(|item| {
			Entry {
				internal_id: item.guid.map_or_else(|| item.link.clone().unwrap(), |guid| guid.value().to_string()),
				url: item
					.link
					.map_or(None, |url| Url::parse(&url).ok()),
				fetch_index: 0, // FIXME
				article_date: SystemTime::now(), // FIXME
				title: item.title.unwrap_or("<untitled>".to_string()),
			}
		}).collect();
		Ok(Feed {
			title,
			entries,
		})
	}
}
