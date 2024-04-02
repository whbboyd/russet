use atom_syndication::Feed as AtomFeed;
use crate::async_util::AsyncUtil;
use crate::feed::model::Entry;
use crate::feed::model::Feed;
use crate::feed::RussetFeedReader;
use crate::Result;
use reqwest::Url;
use std::sync::Arc;
use std::time::SystemTime;

pub struct AtomFeedReader {
	async_util: Arc<AsyncUtil>,
}
impl AtomFeedReader {
	pub fn new(async_util: Arc<AsyncUtil>) -> AtomFeedReader {
		AtomFeedReader{ async_util }
	}
}
impl RussetFeedReader for AtomFeedReader {
	fn load_feed(&self, url: &Url) -> Result<Feed> {
		let content = self.async_util.run_blocking(|| async {
			reqwest::get(url.clone())
				.await?
				.bytes()
				.await
		} )?;
		let atom = AtomFeed::read_from(&content[..])?;
		let title = atom.title.value;
		let entries = atom.entries.into_iter().map(|entry| {
			Entry {
				internal_id: entry.id,
				url: entry
					.links
					.into_iter()
					.filter(|link| { link.rel == "alternate" })
					.next()
					.map_or(None, |url| Url::parse(&url.href).ok()),
				fetch_index: 0, // FIXME
				article_date: SystemTime::now(), // FIXME
				title: entry.title.value,
			}
		}).collect();
		Ok(Feed {
			title,
			url: url.clone(),
			entries,
		})
	}
}
