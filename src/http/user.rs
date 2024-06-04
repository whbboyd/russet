use axum::extract::{ Path, State };
use axum::response::Html;
use crate::http::{ AppState, AuthenticatedUser };
use crate::model::{ UserId, UserType };
use crate::persistence::RussetPersistenceLayer;

#[tracing::instrument]
pub async fn user_page<Persistence>(
	Path(user_id): Path<UserId>,
	State(state): State<AppState<Persistence>>,
	auth_user: AuthenticatedUser<Persistence>,
) -> Html<String>
where Persistence: RussetPersistenceLayer {
	// Authentication rules. Sysops can see all user pages. Members can see only
	// themselves.
	if auth_user.user.user_type != UserType::Sysop && auth_user.user.id != user_id {
		panic!("PERMISSION DENIED!!!1!!");
	}
	let user = state.domain_service.get_user(&user_id).await.unwrap();
	Html(format!("User: {}<br />ID: {:?}<br />Type: {:?}", user.name, user.id, user.user_type))
}
