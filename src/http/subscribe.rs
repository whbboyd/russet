use axum::extract::{ Form, State };
use axum::response::{ Html, Redirect };
use crate::http::AppState;
use crate::http::error::HttpError;
use crate::http::session::AuthenticatedUser;
use crate::model::Timestamp;
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
	generated_time: &'a str,
}
#[tracing::instrument]
pub async fn subscribe_page<Persistence>(
	State(_state): State<AppState<Persistence>>,
	user: AuthenticatedUser<Persistence>,
) -> Result<Html<String>, HttpError>
where Persistence: RussetPersistenceLayer {
	Ok(Html(
		SubscribePage{
			user: Some(&user.user),
			page_title: "Subscribe",
			relative_root: "",
			generated_time: &Timestamp::now().as_iso8601(&user.user.tz),
		}
		.render_once()?
	) )
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
) -> Result<Redirect, HttpError>
where Persistence: RussetPersistenceLayer {
	let url = match Url::parse(&subscribe.url) {
		Ok(url) => url,
		Err(_) => return Err(HttpError::BadRequest { description: format!("Could not parse URL {:?}", subscribe.url) }),
	};
	let feed_id = state.domain_service.add_feed(&url).await?;
	state.domain_service.subscribe(&user.user.id, &feed_id).await?;
	Ok(Redirect::to("/"))
}
