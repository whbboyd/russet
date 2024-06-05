use axum::{ async_trait, RequestPartsExt };
use axum::extract::{ FromRef, FromRequestParts, State };
use axum_extra::extract::cookie::CookieJar;
use axum::http::request::Parts;
use crate::http::AppState;
use crate::http::error::HttpError;
use crate::persistence::model::User;
use crate::persistence::RussetPersistenceLayer;
use std::marker::PhantomData;

#[derive(Debug)]
pub struct AuthenticatedUser<Persistence> {
	pub user: User,
	phantom: PhantomData<Persistence>,
}
#[async_trait]
impl <S, Persistence> FromRequestParts<S> for AuthenticatedUser<Persistence>
where
	S: Send + Sync,
	Persistence: RussetPersistenceLayer,
	AppState<Persistence>: FromRef<S>,
{
	type Rejection = HttpError;
	async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
		let State(state): State<AppState<Persistence>> = State::from_request_parts(parts, state)
			.await
			.expect("Infallible is");
		let cookies = parts.extract::<CookieJar>()
			.await
			.expect("Infallible is");

		let session_cookie = cookies.get("session_id");
		match session_cookie {
			Some(session_cookie) => {
				let user = state.domain_service.auth_user(session_cookie.value()).await?;
				match user {
					Some(user) => Ok(AuthenticatedUser { user, phantom: PhantomData }),
					// Session cookie is present but invalid; user needs to reauthenticate
					None => Err(HttpError::Unauthenticated { redirect_to: None }),
				}
			},
			// Session cookies is missing: user needs to authenticate
			None => Err(HttpError::Unauthenticated { redirect_to: None }),
		}
	}
}
