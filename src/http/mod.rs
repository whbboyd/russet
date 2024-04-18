use axum::extract::State;
use axum::response::{ Html, Redirect };
use axum::Router;
use axum::routing::{ any, get };
use crate::domain::model::Entry;
use crate::domain::RussetDomainService;
use crate::http::session::AuthenticatedUser;
use crate::persistence::model::User;
use crate::persistence::RussetPersistenceLayer;
use sailfish::TemplateOnce;
use std::sync::Arc;

mod session;
mod static_routes;
mod subscribe;
mod login;

pub fn russet_router<Persistence>() -> Router<AppState<Persistence>>
where Persistence: RussetPersistenceLayer {
	Router::new()
		.route("/styles.css", get(static_routes::styles))
		.route("/login", get(login::login_page).post(login::login_user))
		.route("/", get(home))
		.route("/entry/:id", get(|| async { "Entry page!" }))
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
pub struct HomePage<'a> {
	user: &'a User,
	entries: &'a [Entry],
}
#[tracing::instrument]
async fn home<Persistence>(
	State(state): State<AppState<Persistence>>,
	user: AuthenticatedUser<Persistence>,
) -> Html<String>
where Persistence: RussetPersistenceLayer {
	let entries = state.domain_service
		.get_subscribed_entries(&user.user.id)
		.await
		.into_iter()
		.filter_map(|entry| entry.ok())
		.collect::<Vec<Entry>>();
	Html(HomePage{ user: &user.user, entries: entries.as_slice() }.render_once().unwrap())
}

