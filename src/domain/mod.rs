pub mod feeds;
pub mod user;

use argon2::Argon2;
use crate::async_util::AsyncUtil;
use crate::feed::RussetFeedReader;
use crate::persistence::RussetPersistanceLayer;
use crate::Result;
use std::sync::Arc;

pub struct RussetDomainService<'pepper, Persistence>
where Persistence: RussetPersistanceLayer {
	persistence: Persistence,
	readers: Vec<Box<dyn RussetFeedReader>>,
	async_util: Arc<AsyncUtil>,
	password_hash: Argon2<'pepper>,
}
impl <'pepper, Persistence> RussetDomainService<'pepper, Persistence>
where Persistence: RussetPersistanceLayer {
	pub fn new(
		persistence: Persistence,
		readers: Vec<Box<dyn RussetFeedReader>>,
		async_util: Arc<AsyncUtil>,
		pepper: &'pepper [u8],
	) -> Result<RussetDomainService<'pepper, Persistence>> {
		Ok(RussetDomainService {
			persistence,
			readers,
			async_util,
			password_hash: Argon2::new_with_secret(
				&pepper,
				argon2::Algorithm::Argon2id,
				argon2::Version::V0x13,
				argon2::Params::DEFAULT,
			)?,
		} )
	}
}
