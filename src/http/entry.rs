use axum::extract::{ Path, State };
use axum::http::StatusCode;
use axum::response::Redirect;
use crate::http::{ AppState, AuthenticatedUser };
use crate::model::EntryId;
use crate::persistence::RussetPersistenceLayer;
use tracing::error;

#[tracing::instrument]
pub async fn mark_read_redirect<Persistence>(
	Path(entry_id): Path<EntryId>,
	State(state): State<AppState<Persistence>>,
	user: AuthenticatedUser<Persistence>,
) -> Result<Redirect, StatusCode>
where Persistence: RussetPersistenceLayer {
	let entry = state.domain_service.get_entry(&entry_id, &user.user.id).await;
	match entry {
		Ok(entry) => {
			match entry.url {
				Some(url) => Ok(Redirect::to(&url)),
				None => Ok(Redirect::to("/")),
			}
		}
		Err(e) => {
			error!(error = e.as_ref());
			Err(StatusCode::INTERNAL_SERVER_ERROR)
		}
	}
}
