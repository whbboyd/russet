use axum::http::StatusCode;
use axum::response::{ Html, IntoResponse, Response  };
use crate::Err;
use crate::persistence::model::User;
use sailfish::RenderError;
use sailfish::TemplateOnce;
use tracing::error;
use ulid::Ulid;

pub enum HttpError {
	BadRequest { description: String },
	Unauthenticated,
	NotFound,
	InternalError { description: String },
}
impl From<Err> for HttpError {
	fn from(err: Err) -> HttpError {
		HttpError::InternalError { description: err.to_string() }
	}
}
// What even is this doing.
// (The answer: apparently, `?` doesn't like traversing multiple levels of
// `into`? `RenderError` is a `std::Error`, so we can convert it to an `Err`,
// which then converts into an `HttpError`.)
impl From<RenderError> for HttpError {
	fn from(err: RenderError) -> HttpError {
		err.into()
	}
}

#[derive(Debug, TemplateOnce)]
#[template(path = "error.stpl")]
pub struct ErrorPageTemplate<'a> {
	error_code: &'a str,
	error_description: &'a str,
	user: Option<&'a User>,
	page_title: &'a str,
	relative_root: &'a str,
}
impl IntoResponse for HttpError {
	fn into_response(self) -> Response {
		let (status, description) = match self {
			HttpError::BadRequest { description } => (StatusCode::BAD_REQUEST, description),
			HttpError::Unauthenticated => (StatusCode::UNAUTHORIZED, "You must <a href=\"/login\">log in</a>.".to_string()),
			HttpError::NotFound => (StatusCode::NOT_FOUND, "No resource exists at this URL.".to_string()),
			HttpError::InternalError { description } => {
				let correlation_id = Ulid::new();
				let description = format!("Internal server error: {description} ({correlation_id})");
				error!(description);
				// Hide full errors from external users in production.
				// TODO: This should actually be runtime-configurable.
				#[cfg(not(debug_assertions))]
				let description = format!("Internal server error ({correlation_id})");
				(StatusCode::INTERNAL_SERVER_ERROR, description)
			}
		};
		let status_str = format!(
				"{}{}",
				status.as_str(),
				status.canonical_reason()
					.map(|reason| format!(": {reason}"))
					.unwrap_or("".to_string())
			);
		let page = Html(
			ErrorPageTemplate{
					error_code: &status_str,
					error_description: &description,
					user: None,
					page_title: &status_str,
					relative_root: "/",
				}
				.render_once()
				.unwrap_or_else(|err| {
					error!(error = err.to_string(), "An error was encountered rendering the error page");
					format!("{status_str}\n<hr>\nAn error was encountered rendering the error page")
				})
			);
		(status, page).into_response()
	}
}
