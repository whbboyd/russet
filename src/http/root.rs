use axum::extract::{ Form, State };
use axum::http::StatusCode;
use axum::response::{ Html, Redirect };
use crate::domain::model::{ Entry, Feed };
use crate::http::{ AppState, PageQuery };
use crate::http::session::AuthenticatedUser;
use crate::model::{ EntryId, FeedId, Pagination, Timestamp };
use crate::persistence::model::{ User, UserEntry };
use crate::persistence::RussetPersistenceLayer;
use crate::Result;
use sailfish::TemplateOnce;
use std::collections::HashMap;
use std::time::SystemTime;
use tracing::error;

// Root (home/entries) page template
#[derive(TemplateOnce)]
#[template(path = "root.stpl")]
struct RootPageTemplate<'a> {
	user: Option<&'a User>,
	entries: &'a [Entry],
	feeds: &'a HashMap<FeedId, Feed>,
	page_num: usize,
	page_title: &'a str,
	relative_root: &'a str,
}
#[tracing::instrument]
pub async fn root<Persistence>(
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
		RootPageTemplate {
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

#[derive(Debug)]
enum Action {
	MarkRead,
	Delete,
}
#[derive(Debug)]
pub struct EditUserEntriesRequest {
	action: Action,
	select_all: bool,
	selected_ids: Vec<EntryId>,
}
impl EditUserEntriesRequest {
	fn from_raw_entries(entries: &Vec<(String, String)>) -> Result<EditUserEntriesRequest> {
		let mut action: Option<Action> = None;
		let mut select_all = false;
		let mut selected_ids: Vec<EntryId> = Vec::new();
		for (key, value) in entries {
			match key.as_str() {
				"action" => {
					if action.is_some() {
						return Err(format!("Multiple actions: at least {action:?} and {value:?}").into())
					}
					action = Some(match value.as_str() {
						"mark_read" => Action::MarkRead,
						"delete" => Action::Delete,
						_ => return Err(format!("").into()),
					});
				},
				"select-all" => select_all = true,
				key if key.starts_with("select-") => {
					let suffix = key.strip_prefix("select-").unwrap();
					let id = ulid::Ulid::from_string(suffix)?;
					selected_ids.push(EntryId(id));
				},
				_ => return Err(format!("Bad key: {key:?}").into()),
			}
		}
		let action = action.ok_or(Into::<crate::Err>::into("No action"))?;
		Ok(EditUserEntriesRequest{
			action,
			select_all,
			selected_ids,
		})
	}
}
pub async fn edit_userentries<Persistence>(
	State(state): State<AppState<Persistence>>,
	user: AuthenticatedUser<Persistence>,
	Form(request): Form<Vec<(String, String)>>,
) -> std::result::Result<Redirect, StatusCode>
where Persistence: RussetPersistenceLayer {
	let request = EditUserEntriesRequest::from_raw_entries(&request)
		.map_err(|err| {
			error!("{err:?}");
			StatusCode::INTERNAL_SERVER_ERROR
		})?;
	let time = Some(Timestamp::new(SystemTime::now()));
	let user_entry = match request.action {
		Action::MarkRead => UserEntry { read: time, tombstone: None },
		Action::Delete => UserEntry { read: time.clone(), tombstone: time },
	};
	state.domain_service.set_userentries(&request.selected_ids, &user.user.id, &user_entry)
		.await
		.map_err(|err| {
			error!("{err:?}");
			StatusCode::INTERNAL_SERVER_ERROR
		})?;
	Ok(Redirect::to("/"))
}
