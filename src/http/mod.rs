use axum::extract::{ Form, State };
use axum::response::{ Html, Redirect };
use axum::Router;
use axum::routing::{ any, get, post };
use crate::domain::model::{ Entry, Feed };
use crate::domain::RussetDomainService;
use crate::http::session::AuthenticatedUser;
use crate::model::{ FeedId, Pagination };
use crate::persistence::model::User;
use crate::persistence::RussetPersistenceLayer;
use sailfish::TemplateOnce;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tower::limit::GlobalConcurrencyLimitLayer;

mod entry;
mod feed;
mod login;
mod session;
mod static_routes;
mod subscribe;

pub fn russet_router<Persistence>(
	global_concurrent_limit: u32,
	login_concurrent_limit: u32,
) -> Router<AppState<Persistence>>
where Persistence: RussetPersistenceLayer {
	let global_limit_semaphore = Arc::new(Semaphore::new(global_concurrent_limit.try_into().unwrap()));
	let login_limit_sempahore = Arc::new(Semaphore::new(login_concurrent_limit.try_into().unwrap()));
	Router::new()
		.route("/login", post(login::login_user))
		.layer(GlobalConcurrencyLimitLayer::with_semaphore(login_limit_sempahore))
		.route("/login", get(login::login_page))
		.route("/styles.css", get(static_routes::styles))
		.route("/", get(home))
		.route("/entry/:id", get(entry::mark_read_redirect))
		.route("/feed/:id", get(feed::feed_page).post(feed::unsubscribe))
		.route("/subscribe", get(subscribe::subscribe_page).post(subscribe::subscribe))
		.route("/*any", any(|| async { Redirect::to("/") }))
		.layer(GlobalConcurrencyLimitLayer::with_semaphore(global_limit_semaphore))
}

#[derive(Debug)]
pub struct AppState<Persistence>
where Persistence: RussetPersistenceLayer {
	pub domain_service: Arc<RussetDomainService<Persistence>>,
}
impl <Persistence> Clone for AppState<Persistence>
where Persistence: RussetPersistenceLayer {
	fn clone(&self) -> Self { AppState { domain_service: self.domain_service.clone() } }
}

#[derive(TemplateOnce)]
#[template(path = "home.stpl")]
struct HomePageTemplate<'a> {
	user: Option<&'a User>,
	entries: &'a [Entry],
	feeds: &'a HashMap<FeedId, Feed>,
	page_num: usize,
	page_title: &'a str,
	relative_root: &'a str,
}
#[derive(Debug, Deserialize)]
struct PageQuery {
	page_num: Option<usize>,
	page_size: Option<usize>,
}
#[tracing::instrument]
async fn home<Persistence>(
	State(state): State<AppState<Persistence>>,
	user: AuthenticatedUser<Persistence>,
	Form(pagination): Form<PageQuery>,
) -> Html<String>
where Persistence: RussetPersistenceLayer {
	let page_num = pagination.page_num.unwrap_or(0);
	let page_size = pagination.page_size.unwrap_or(100);
	let pagination = Pagination { page_num, page_size };
	let entries = state.domain_service
		.get_subscribed_entries(&user.user.id, &pagination)
		.await
		.into_iter()
		.filter_map(|entry| entry.ok())
		.collect::<Vec<Entry>>();
	let feeds = state.domain_service
		.feeds_for_user(&user.user.id)
		.await
		.into_iter()
		.filter_map(|feed| feed.ok())
		.map(|feed| (feed.id.clone(), feed))
		.collect::<HashMap<FeedId, Feed>>();
	Html(
		HomePageTemplate {
			user: Some(&user.user),
			entries: entries.as_slice(),
			feeds: &feeds,
			page_num: pagination.page_num,
			page_title: "Entries",
			relative_root: "",
		}
		.render_once()
		.unwrap()
	)
}

