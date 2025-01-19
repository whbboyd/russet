mod update;

use crate::domain::model::Feed;
use crate::domain::RussetDomainService;
use crate::Result;
use crate::model::{ EntryId, FeedId, UserId, Timestamp };
use crate::persistence::model::{ Entry, Feed as PersistenceFeed, FeedCheck, WriteFeedCheck };
use crate::persistence::{ RussetEntryPersistenceLayer, RussetFeedPersistenceLayer };
use crate::feed::model::Feed as ReaderFeed;
use reqwest::header::{ ETAG, IF_NONE_MATCH };
use reqwest::StatusCode;
use reqwest::Url;
use std::collections::HashSet;
use ulid::Ulid;

impl <Persistence> RussetDomainService<Persistence>
where Persistence: RussetEntryPersistenceLayer + RussetFeedPersistenceLayer {

	/// Update the stored entries for the given feed.
	///
	/// Returns the [FeedCheck] generated from this update.
	pub async fn update_feed(&self, feed_id: &FeedId, last_check: &CheckState)
		-> Result<FeedCheck>
	{
		let feed = self.persistence.get_feed(feed_id).await?;

		// Fetch the feed data. We do this now (before recording the check)
		// because some of its details will need to feed back into the check.
		let fetch_response = self.fetch(&feed.url, last_check.etag()).await?;

		// Now, generate the check. We need this to store the entries, because
		// they must be tagged with the check that generated them.
		let check = self
			.build_check_and_update(last_check, feed_id, &fetch_response)
			.await?;

		Ok(check)
	}

	/// Get all feeds known to the persistence layer
	pub async fn get_feeds(&self) -> impl IntoIterator<Item = Result<Feed>> + '_ {
		self.persistence
			.get_feeds()
			.await
			.into_iter()
			.map(|feed| {
				feed.map(|feed| { feed.into() } )
			} )
	}

	/// Given a URL, ensure the feed is stored in the persistence layer.
	///
	/// If a feed with that URL is already stored, no action is taken.
	/// Otherwise, the feed will be downloaded and added to the persistence
	/// layer.
	pub async fn add_feed(&self, url: &Url) -> Result<FeedId> {
		match self.persistence.get_feed_by_url(url).await? {
			Some(feed) => {
				// TODO: Contemplate rescheduling the next check in this case.
				Ok(feed.id)
			}
			None => {
				let fetch_response = self.fetch(url, &None).await?;
				match &fetch_response {
					FetchResponse::Feed(reader_feed) => {
						let feed = PersistenceFeed {
							id: FeedId(Ulid::new()),
							title: reader_feed.title.clone(),
							url: url.clone(),
						};
						self.persistence.add_feed(&feed).await?;
						// TODO: Add the feed to scheduling.
						self.build_check_and_update(
								&CheckState::NoCheck(Timestamp::now()),
								&feed.id,
								&fetch_response,
							).await?;

						Ok(feed.id)
					}
					_ => Err(format!("Error on initial feed fetch: {:?}; not adding feed", fetch_response).into())
				}
			}
		}
	}

	pub async fn feeds_for_user(&self, user_id: &UserId) -> Vec<Result<Feed>> {
		self.persistence
			.get_subscribed_feeds(user_id)
			.await
			.into_iter()
			.map(|feed| {
				feed.map(|feed| { feed.into() } )
			} )
			.collect()
	}

	pub async fn get_feed(&self, feed_id: &FeedId) -> Result<Feed> {
		self.persistence
			.get_feed(feed_id)
			.await
			.map(|feed| { feed.into() } )
	}

	pub async fn get_last_feed_check(&self, feed_id: &FeedId)
		-> Result<Option<FeedCheck>>
	{
		self.persistence
			.get_last_feed_check(feed_id)
			.await
	}


	/// Fetch feed data from the remote system
	async fn fetch(&self, url: &Url, etag: &Option<String>) -> Result<FetchResponse> {
		let mut request = self.http_client
				.get(url.clone());
		if let Some(etag) = etag { request = request.header(IF_NONE_MATCH, etag) }

		let response = request.send().await?;

		// Special cases to check.

		// Not modified (304): our etag matched, no updates.
		if response.status() == StatusCode::NOT_MODIFIED {
			return Ok(FetchResponse::EtagMatch)
		}
		// TODO: cache control indicated no updates
		// Client error: probably something is wrong with our feed record. Under
		// most circumstances these won't recover, but servers always lie, so we
		// won't assume that.
		if response.status().is_client_error() {
			return Ok(FetchResponse::ClientError(response.status()))
		}
		// Server error. Under most circumstances these will eventually recover.
		// (But servers always lie.)
		if response.status().is_server_error() {
			return Ok(FetchResponse::ServerError(response.status()))
		}

		// Otherwise, we have a real response. Pull out the headers we're
		// interested in and attempt to parse the body.
		let etag = response
				.headers()
				.get(ETAG)
				// If some jerk sends us an etag with invalid UTF-8, we'll just
				// drop it.
				.and_then(|value| value.to_str().map(|str| str.to_string()).ok());
		let bytes = response.bytes().await?;
		// TODO: Store a reader hint with the feed to save redundant parsing effort
		let mut reader_feed = self.feed_from_bytes(&bytes).await?;
		reader_feed.etag = etag;
		Ok(FetchResponse::Feed(reader_feed))
	}

	/// Given a parsed `reader_feed`, generate and persist a [FeedCheck] for it
	/// and update the persistence layer with its entries
	async fn build_check_and_update(
		&self,
		check_state: &CheckState,
		feed_id: &FeedId,
		fetch_response: &FetchResponse,
	) -> Result<FeedCheck> {
		// Generate the check. We need this to store the entries, because
		// they must be tagged with the check that generated them.
		// TODO: Whole lotta logic goes here:
		let next_check_time = match fetch_response {
			FetchResponse::Feed(_) | FetchResponse::EtagMatch => *check_state.check_time() + self.default_feed_check_interval,
			FetchResponse::ClientError(_) => *check_state.check_time() + self.max_feed_check_interval,
			FetchResponse::ServerError(_) => *check_state.check_time() + self.default_feed_check_interval,
		};
		let etag = match fetch_response {
			FetchResponse::Feed(feed) => feed.etag.clone(),
			FetchResponse::EtagMatch => check_state.etag().clone(),
			_ => None,
		};

		let check = self.persistence.add_feed_check(WriteFeedCheck {
			feed_id: feed_id.clone(),
			check_time: check_state.check_time().clone(),
			next_check_time,
			etag,
		} ).await?;

		// Finally, store the entries, tagged with the check.
		if let FetchResponse::Feed(reader_feed) = fetch_response {
			self.update_with_entries(feed_id, &reader_feed, check.id).await?;
		}

		Ok(check)
	}

	/// Given a parsed `reader_feed`, update the persistence layer for the given
	/// feed with the entries from it
	async fn update_with_entries(&self, feed_id: &FeedId, reader_feed: &ReaderFeed, check_id: u64) -> Result<()> {
		let known_internal_ids = self.persistence
			.get_entries_for_feed(&feed_id)
			.await
			.into_iter()
			.filter_map(|entry| entry.ok().map(|entry| entry.internal_id) )
			.collect::<HashSet<String>>();
		let new_entries = reader_feed.entries.as_slice().into_iter()
			.filter(|entry| !known_internal_ids.contains(&entry.internal_id) )
			.map (|entry| {
				Entry {
					id: EntryId(Ulid::new()),
					feed_id: feed_id.clone(),
					internal_id: entry.internal_id.clone(),
					check_id,
					article_date: entry.article_date.clone(),
					title: entry.title.clone(),
					url: entry.url.clone(),
				}
			} )
			.collect::<Vec<Entry>>();
		for e in new_entries.as_slice() {
			self.persistence.add_entry(e, &feed_id).await?;
		}
		Ok(())
	}

	/// Given a serialized feed, attempt to deserialize it using all the known
	/// `readers`.
	///
	/// TODO: This always attempts all `readers`, but a given URL will virtually
	/// never change format. We should store a format hint with the feed and
	/// optimistically try that format first, before falling back to others.
	async fn feed_from_bytes(&self, bytes: &[u8]) -> Result<ReaderFeed> {
		let mut acc = Vec::new();
		for reader in self.readers.as_slice() {
			acc.push(async {
				reader.read_feed(&bytes)
			} );
		}
		let mut parsed_feed: Option<ReaderFeed> = None;
		for future in acc {
			if parsed_feed.is_some() {
				drop(future);
			} else if let Ok(feed) = future.await {
				parsed_feed = Some(feed);
			}
		}
		parsed_feed.ok_or_else(|| "Unable to load feed".into())
	}
}

/// Previous check information.
///
/// We might have a previous [FeedCheck], in which case we should use its data.
/// But we also might not (for example, on initial update of a feed). In that
/// case, we still have *some* data—for instance, the scheduled time—but not a
/// full check object.
#[derive(Debug)]
pub enum CheckState {
	Check(FeedCheck),
	NoCheck(Timestamp),
}
impl CheckState {
	pub fn check_time(&self) -> &Timestamp {
		match self {
			CheckState::Check(check) => &check.next_check_time,
			CheckState::NoCheck(check_time) => check_time,
		}
	}
	pub fn etag(&self) -> &Option<String> {
		match self {
			CheckState::Check(check) => &check.etag,
			CheckState::NoCheck(_) => &None,
		}
	}
}

/// Fetch result.
#[derive(Debug)]
enum FetchResponse {
	Feed(ReaderFeed),
	EtagMatch,
	#[allow(dead_code)]
	ClientError(StatusCode),
	#[allow(dead_code)]
	ServerError(StatusCode),
}

