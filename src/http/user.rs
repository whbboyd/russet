use axum::extract::{ Path, State };
use axum::response::Html;
use crate::http::{ AppState, AuthenticatedUser };
use crate::http::error::HttpError;
use crate::model::{ UserId, UserType };
use crate::persistence::RussetPersistenceLayer;
use crate::persistence::model::User;
use sailfish::TemplateOnce;

#[derive(Clone, Debug, TemplateOnce)]
#[template(path = "user.stpl")]
pub struct UserPage<'a> {
	page_user: &'a User,
	user: Option<&'a User>,
	page_title: &'a str,
	relative_root: &'a str,
}
#[tracing::instrument]
pub async fn user_page<Persistence>(
	Path(page_user_id): Path<UserId>,
	State(state): State<AppState<Persistence>>,
	auth_user: AuthenticatedUser<Persistence>,
) -> Result<Html<String>, HttpError>
where Persistence: RussetPersistenceLayer {
	// Authentication rules. Sysops can see all user pages. Members can see only
	// themselves.
	if auth_user.user.user_type != UserType::Sysop &&
			auth_user.user.id != page_user_id {
		return Err(HttpError::Forbidden);
	}
	let page_user = state.domain_service.get_user(&page_user_id).await?;
	let page_title = format!("User - {}", page_user.name);
	Ok(Html(
		UserPage{
			page_user: &page_user,
			user: Some(&auth_user.user),
			page_title: &page_title,
			relative_root: "../",
		}
		.render_once()?
	) )
}
