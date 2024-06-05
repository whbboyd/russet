use axum::extract::{ Form, State };
use axum_extra::extract::cookie::{ Cookie, CookieJar, Expiration };
use axum::response::{ Html, Redirect };
use crate::http::AppState;
use crate::http::error::HttpError;
use crate::persistence::RussetPersistenceLayer;
use sailfish::TemplateOnce;
use serde::Deserialize;

#[derive(Debug, TemplateOnce)]
#[template(path = "login.stpl")]
pub struct LoginPageTemplate<'a> {
	redirect_to: Option<&'a str>,
	page_title: &'a str,
	relative_root: &'a str,
	user: Option<&'a crate::persistence::model::User>,
}
#[derive(Debug, Deserialize)]
pub struct LoginPageQuery {
	redirect_to: Option<String>,
}
#[tracing::instrument]
pub async fn login_page<Persistence>(
	State(_state): State<AppState<Persistence>>,
	Form(login): Form<LoginPageQuery>,
) -> Result<Html<String>, HttpError>
where Persistence: RussetPersistenceLayer {
	Ok(Html(
		LoginPageTemplate{
			redirect_to: login.redirect_to.as_ref().map(|redirect| redirect.as_str()),
			page_title: "Login",
			relative_root: "",
			user: None,
		}
		.render_once()?
	) )
}

#[derive(Deserialize, Clone)]
pub struct LoginRequest {
	user_name: String,
	plaintext_password: String,
	redirect_to: Option<String>,
	#[serde(default = "default_permanent_session")]
	permanent_session: bool,
}
// This is dumb.
fn default_permanent_session() -> bool { false }
impl std::fmt::Debug for LoginRequest {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("LoginRequest")
			.field("user_name", &self.user_name)
			.field("plaintext_password", &"<redacted>")
			.field("redirect_to", &self.redirect_to)
			.field("permanent_session", &self.permanent_session)
			.finish()
	}
}
#[tracing::instrument]
pub async fn login_user<Persistence>(
	State(state): State<AppState<Persistence>>,
	cookies: CookieJar,
	Form(login): Form<LoginRequest>,
) -> Result<(CookieJar, Redirect), HttpError>
where Persistence: RussetPersistenceLayer {
	let session = state.domain_service
		.login_user(
			login.user_name,
			login.plaintext_password,
			login.permanent_session,
		)
		.await?;
	match session {
		Some(session) => {
			let cookie = Cookie::build(("session_id", session.token.0))
				.expires(
					if login.permanent_session {
						Expiration::DateTime(session.expiration.0.into())
					} else {
						Expiration::Session
					}
				)
				.build();
			Ok((
				cookies.add(cookie),
				Redirect::to(&login.redirect_to.unwrap_or("/".to_string())),
			))
		},
		None => Err(HttpError::Unauthenticated { redirect_to: login.redirect_to }),
	}
}
