use atom_syndication::Feed as AtomFeed;
use crate::feed::model::Entry;
use crate::feed::model::Feed;
use crate::feed::RussetFeedReader;
use crate::Result;
use reqwest::Url;

#[derive(Debug)]
pub struct AtomFeedReader { }
impl AtomFeedReader {
	pub fn new() -> AtomFeedReader {
		AtomFeedReader{ }
	}
}
impl RussetFeedReader for AtomFeedReader {

	fn read_feed(&self, bytes: &[u8]) -> Result<Feed> {
		let atom = AtomFeed::read_from(bytes)?;
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
				article_date: entry.updated.into(),
				title: entry.title.value,
			}
		}).collect();
		Ok(Feed {
			title,
			entries,
		})
	}
}
