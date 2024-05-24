use axum::extract::{ Form, Path, State };
use axum::http::StatusCode;
use axum::response::{ Html, Redirect };
use crate::domain::model::{ Entry, Feed };
use crate::http::{ AppState, AuthenticatedUser, PageQuery };
use crate::model::{ FeedId, Pagination };
use crate::persistence::model::User;
use crate::persistence::RussetPersistenceLayer;
use sailfish::TemplateOnce;

#[derive(TemplateOnce)]
#[template(path = "feed.stpl")]
struct FeedPageTemplate<'a> {
	user: Option<&'a User>,
	entries: &'a [Entry],
	feed: &'a Feed,
	page_num: usize,
	page_title: &'a str,
	relative_root: &'a str,
}
#[tracing::instrument]
pub async fn feed_page<Persistence>(
	Path(feed_id): Path<FeedId>,
	State(state): State<AppState<Persistence>>,
	user: AuthenticatedUser<Persistence>,
	Form(pagination): Form<PageQuery>,
) -> Html<String>
where Persistence: RussetPersistenceLayer {
	let page_num = pagination.page_num.unwrap_or(0);
	let page_size = pagination.page_size.unwrap_or(100);
	let pagination = Pagination { page_num, page_size };
	let feed = state.domain_service.get_feed(&feed_id).await.unwrap();
	let entries = state.domain_service
		.get_feed_entries(&user.user.id, &feed_id, &pagination)
		.await
		.into_iter()
		.filter_map(|entry| entry.ok())
		.collect::<Vec<Entry>>();
	let page_title = format!("Feed - {}", feed.title);
	Html(
		FeedPageTemplate {
			user: Some(&user.user),
			entries: &entries.as_slice(),
			feed: &feed,
			page_num: pagination.page_num,
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
