pub mod feeds;

use crate::async_util::AsyncUtil;
use crate::feed::RussetFeedReader;
use crate::persistence::RussetPersistanceLayer;
use std::sync::Arc;

pub struct RussetDomainService<Persistence>
where Persistence: RussetPersistanceLayer {
	persistence: Persistence,
	readers: Vec<Box<dyn RussetFeedReader>>,
	async_util: Arc<AsyncUtil>,
}
impl <Persistence> RussetDomainService<Persistence>
where Persistence: RussetPersistanceLayer {
	pub fn new(
		persistence: Persistence,
		readers: Vec<Box<dyn RussetFeedReader>>,
		async_util: Arc<AsyncUtil>,
	) -> RussetDomainService<Persistence> {
		RussetDomainService {
			persistence,
			readers,
			async_util,
		}
	}
}
