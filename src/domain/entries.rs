use crate::domain::RussetDomainService;
use crate::persistence::model::{ Entry, UserId };
use crate::persistence::RussetEntryPersistenceLayer;
use crate::Result;

impl <Persistence> RussetDomainService<Persistence>
where Persistence: RussetEntryPersistenceLayer {
	pub async fn get_subscribed_entries(&self, user_id: &UserId) -> impl IntoIterator<Item = Result<Entry>> {
		self.persistence.get_entries_for_user(user_id).await.into_iter().collect::<Vec<Result<Entry>>>()
	}
}
