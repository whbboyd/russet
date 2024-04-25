use axum::extract::{ Form, State };
use axum::response::{ Html, Redirect };
use axum::Router;
use axum::routing::{ any, get };
use crate::domain::model::Entry;
use crate::domain::RussetDomainService;
use crate::http::session::AuthenticatedUser;
use crate::model::Pagination;
use crate::persistence::model::User;
use crate::persistence::RussetPersistenceLayer;
use sailfish::TemplateOnce;
use serde::Deserialize;
use std::sync::Arc;

mod entry;
mod login;
mod session;
mod static_routes;
mod subscribe;

pub fn russet_router<Persistence>() -> Router<AppState<Persistence>>
where Persistence: RussetPersistenceLayer {
	Router::new()
		.route("/styles.css", get(static_routes::styles))
		.route("/login", get(login::login_page).post(login::login_user))
		.route("/", get(home))
		.route("/entry/:id", get(entry::mark_read_redirect))
		.route("/feed/:id", get(|| async { "Feed page!" }))
		.route("/subscribe", get(subscribe::subscribe_page).post(subscribe::subscribe))
		.route("/*any", any(|| async { Redirect::to("/") }))
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
	user: &'a User,
	entries: &'a [Entry],
	page_num: usize,
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
	Html(
		HomePageTemplate{
			user: &user.user,
			entries: entries.as_slice(),
			page_num: pagination.page_num
		}
		.render_once()
		.unwrap()
	)
}

