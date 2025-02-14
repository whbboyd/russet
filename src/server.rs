use crate::Result;
use crate::domain::RussetDomainService;
use crate::http::{ AppState, russet_router };
use crate::model::{ FeedId, Timestamp };
use crate::persistence::RussetPersistenceLayer;
use std::sync::Arc;
use std::time::Duration;
use tracing::{ error, info };
use tokio::select;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;

const SESSION_CLEANUP_INTERVAL: Duration = Duration::from_secs(3_600);

/// Start the Russet server.
///
/// This sets up background tasks like scheduled feed checks and session
/// cleanup, and then serves requests (see [russet_router]) until an exit signal
/// is received, at which point it cancels background tasks and then returns.
pub async fn start<Persistence>(
	domain_service: Arc<RussetDomainService<Persistence>>,
	listen: String,
	global_concurrent_limit: u32,
	login_concurrent_limit: u32,
) -> Result<()>
where Persistence: RussetPersistenceLayer {
	info!("Starting {}…", crate::APP_NAME);
	let mut tasks = vec![];
	let task_tracker = TaskTracker::new();

	// Start the feed update coroutines
	for feed in domain_service.get_feeds().await {
		match feed {
			Ok(feed) => {
				tasks.push(feed_check(feed.id, domain_service.clone(), task_tracker.clone()).await);
			}
			Err(err) => {
				error!(error = err, "Error loading feed, skipping")
			}
		}
	}

	// Start the expired session cleanup coroutine
	tasks.push(session_cleanup(domain_service.clone(), task_tracker.clone()).await);

	// Start the HTTP server
	let app_state = AppState { domain_service: domain_service.clone() };
	let routes = russet_router(global_concurrent_limit, login_concurrent_limit)
		.with_state(app_state);
	let listener = tokio::net::TcpListener::bind(&listen).await?;
	let graceful_exit_signal = async {
		tokio::signal::ctrl_c().await.expect("Failed to register interrupt handler");
		info!("Received interrupt, exiting…");
	};
	info!("Initialization complete, serving requests!");
	info!("Listening on {listen}…");
	axum::serve(listener, routes)
		.with_graceful_shutdown(graceful_exit_signal)
		.await?;

	info!("Exiting, waiting for tasks to complete…");
	for task in tasks {
		task.cancel();
	}
	task_tracker.close();
	task_tracker.wait().await;
	Ok(())
}

/// Schedule a coroutine to check the given feed at appropriate intervals.
///
/// The returned [CancellationToken] can be used to cancel the coroutine, and
/// the corouting will be registered with [task_tracker] so its exit can be
/// joined on.
async fn feed_check<Persistence>(
	feed_id: FeedId,
	domain_service: Arc<RussetDomainService<Persistence>>,
	task_tracker: TaskTracker,
) -> CancellationToken
where Persistence: RussetPersistenceLayer {
	let token = CancellationToken::new();
	let captured_token = token.clone();
	task_tracker.spawn(async move {
		// Initial wait. Pull the latest check; if there is one and it has a
		// future next check time, wait until then.
		let check = domain_service.get_last_feed_check(&feed_id).await;
		let now = Timestamp::now();
		let mut check_time = match check {
			Ok(None) => now,
			Ok(Some(check)) => {
				// If the scheduled next check is in the past, we missed it.
				// Don't wait, and the check time is now.
				if check.next_check_time < now {
					now
				} else {
					if let WaitResult::Cancellation = wait_until(
						check.next_check_time,
						&captured_token,
					).await {
						return
					}
					check.next_check_time
				}
			}
			Err(err) => {
				let next_check = Timestamp::now() + domain_service.default_feed_check_interval;
				error!(error = err, "Error determining next check time for feed {feed_id:?}; scheduling next check for {next_check:?}");
				if let WaitResult::Cancellation = wait_until(
					next_check,
					&captured_token,
				).await {
					return
				}
				next_check
			}
		};
		loop {
			// Perform the check
			info!("Checking for updates to {feed_id:?}");
			check_time = match domain_service.update_feed(&feed_id, &check_time).await {
				Ok(check) => check.next_check_time,
				Err(err) => {
					let next_check = Timestamp::now() + domain_service.default_feed_check_interval;
					error!(error = err, "Error performing check for feed {feed_id:?}; scheduling next check for {next_check:?}");
					next_check
				}
			};

			// Wait for either next scheduled check or cancellation
			if let WaitResult::Cancellation = wait_until(
				check_time,
				&captured_token,
			).await {
				return
			}
		}
	} );
	token
}

/// Schedule a coroutine to remove expired sessions from the persistence layer.
///
/// The returned [CancellationToken] can be used to cancel the coroutine, and
/// the corouting will be registered with [task_tracker] so its exit can be
/// joined on.
async fn session_cleanup<Persistence>(
	domain_service: Arc<RussetDomainService<Persistence>>,
	task_tracker: TaskTracker,
) -> CancellationToken
where Persistence: RussetPersistenceLayer {
	let token = CancellationToken::new();
	let captured_token = token.clone();
	task_tracker.spawn(async move {
		loop {
			info!("Removing expired sessions");
			if let Err(e) = domain_service.cleanup_expired_sessions().await {
				error!(error = e.as_ref(), "Error removing expired sessions");
			}
			if let WaitResult::Cancellation = wait_until(
				Timestamp::now() + SESSION_CLEANUP_INTERVAL,
				&captured_token,
			).await {
				return
			}
		}
	} );
	token
}

/// Wait [until] the given timestamp, or until [token] is cancelled.
async fn wait_until(until: Timestamp, token: &CancellationToken) -> WaitResult {
	let delay = Timestamp::until(until);
	select! {
		_ = tokio::time::sleep(delay) => { return WaitResult::Timeout }
		_ = token.cancelled() => { return WaitResult::Cancellation }
	}
}
enum WaitResult {
	Timeout,
	Cancellation,
}
