use std::future::Future;
use tokio::runtime::Runtime;
use tokio::sync::oneshot;

/// Ugh.
///
/// We're using some async libraries. Russet is synchronous. This helps
/// bridge the gap. We maintain an internal Tokio runtime ([runtime]; ugh), use
/// it to execute the task, and use a [oneshot] channel to block on task
/// completion and get the return value out of it.
#[derive(Debug)]
pub struct AsyncUtil {
	runtime: Runtime,
}
impl AsyncUtil {
	pub fn new() -> AsyncUtil {
		AsyncUtil {
			runtime: Runtime::new().unwrap(),
		}
	}
	pub fn run_blocking<T, F, R>(&self, mut fun: F) -> T
	where F: FnMut() -> R, R: Future<Output = T> {
		let (tx, rx) = oneshot::channel::<T>();
		self.runtime.block_on(async {
			tx.send(fun().await)
		} ).ok().unwrap();
		rx.blocking_recv().unwrap()
	}
}
