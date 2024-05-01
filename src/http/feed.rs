use axum::extract::{ Path, State };
use axum::http::StatusCode;
use axum::response::{ Html, Redirect };
use crate::domain::model::Feed;
use crate::http::{ AppState, AuthenticatedUser };
use crate::model::FeedId;
use crate::persistence::model::User;
use crate::persistence::RussetPersistenceLayer;
use sailfish::TemplateOnce;

#[derive(TemplateOnce)]
#[template(path = "feed.stpl")]
struct FeedPageTemplate<'a> {
	user: Option<&'a User>,
	feed: &'a Feed,
	page_title: &'a str,
	relative_root: &'a str,
}
#[tracing::instrument]
pub async fn feed_page<Persistence>(
	Path(feed_id): Path<FeedId>,
	State(state): State<AppState<Persistence>>,
	user: AuthenticatedUser<Persistence>,
) -> Html<String>
where Persistence: RussetPersistenceLayer {
	let feed = state.domain_service.get_feed(&feed_id).await.unwrap();
	let page_title = format!("Feed - {}", feed.title);
	Html(
		FeedPageTemplate {
			user: Some(&user.user),
			feed: &feed,
			page_title: &page_title,
			relative_root: "../",
		}
		.render_once()
		.unwrap()
	)
}

#[tracing::instrument]
pub async fn unsubscribe<Persistence>(
	Path(feed_id): Path<FeedId>,
	State(state): State<AppState<Persistence>>,
	user: AuthenticatedUser<Persistence>,
) -> Result<Redirect, StatusCode>
where Persistence: RussetPersistenceLayer {
	state.domain_service.unsubscribe(&user.user.id, &feed_id).await.unwrap();
	Ok(Redirect::to("../"))
}
