use crate::domain::RussetDomainService;
use crate::{ Err, Result };
use crate::persistence::model::{ Entry, Feed };
use crate::persistence::RussetPersistanceLayer;
use crate::feed::model::Feed as ReaderFeed;
use reqwest::Url;
use ulid::Ulid;

impl <'pepper, Persistence> RussetDomainService<'pepper, Persistence>
where Persistence: RussetPersistanceLayer {

	pub fn login_user(user_name: String, plaintext_password: String) -> Option<String> {
		todo!()
	}
}
