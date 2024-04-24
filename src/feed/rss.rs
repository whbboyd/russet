use chrono::format::{ Item, parse, Parsed };
use chrono::format::Fixed::RFC2822;
use crate::feed::model::Entry;
use crate::feed::model::Feed;
use crate::feed::RussetFeedReader;
use crate::model::Timestamp;
use crate::Result;
use reqwest::Url;
use rss::Channel;
use std::time::SystemTime;

#[derive(Debug)]
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
				article_date: from_rss_timestamp(item.pub_date),
				title: item.title.unwrap_or("<untitled>".to_string()),
			}
		}).collect();
		Ok(Feed {
			title,
			entries,
		})
	}
}

const RFC_2822: [Item; 1] = [Item::Fixed(RFC2822)];
fn from_rss_timestamp(ts: Option<String>) -> Timestamp {
	if let Some(ts) = || -> Option<SystemTime> {
		let ts = ts?;
		let mut parsed = Parsed::new();
		parse(&mut parsed, &ts, RFC_2822.iter()).ok()?;
		Some(parsed.to_datetime().ok()?.into())
	}() {
		Timestamp::new(ts)
	} else {
		Timestamp::new(SystemTime::now())
	}
}
