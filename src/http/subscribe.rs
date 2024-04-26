use axum::extract::{ Form, State };
use axum::http::StatusCode;
use axum::response::{ Html, Redirect };
use crate::http::AppState;
use crate::http::session::AuthenticatedUser;
use crate::persistence::model::User;
use crate::persistence::RussetPersistenceLayer;
use reqwest::Url;
use sailfish::TemplateOnce;
use serde::Deserialize;

#[derive(Clone, Debug, TemplateOnce)]
#[template(path = "subscribe.stpl")]
pub struct SubscribePage<'a> {
	user: Option<&'a User>,
	page_title: &'a str,
	relative_root: &'a str,
}
#[tracing::instrument]
pub async fn subscribe_page<Persistence>(
	State(_state): State<AppState<Persistence>>,
	user: AuthenticatedUser<Persistence>,
) -> Html<String>
where Persistence: RussetPersistenceLayer {
	Html(
		SubscribePage{
			user: Some(&user.user),
			page_title: "Subscribe",
			relative_root: "",
		}
		.render_once()
		.unwrap()
	)
}

#[derive(Debug, Deserialize, Clone)]
pub struct SubscribeRequest {
	url: String,
}
#[tracing::instrument]
pub async fn subscribe<Persistence>(
	State(state): State<AppState<Persistence>>,
	user: AuthenticatedUser<Persistence>,
	Form(subscribe): Form<SubscribeRequest>,
) -> Result<Redirect, StatusCode>
where Persistence: RussetPersistenceLayer {
	let url = match Url::parse(&subscribe.url) {
		Ok(url) => url,
		Err(_) => return Err(StatusCode::BAD_REQUEST),
	};
	let feed_id = match state.domain_service.add_feed(&url).await {
		Ok(feed_id) => feed_id,
		Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
	};
	match state.domain_service.subscribe(&user.user.id, &feed_id).await {
		Ok(_) => (),
		Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
	};
	Ok(Redirect::to("/"))
}
