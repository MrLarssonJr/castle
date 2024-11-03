use std::sync::{Arc, RwLock};
use tokio::sync::Notify;

#[derive(Debug, Clone)]
pub struct Watch<T> {
	value: Arc<RwLock<Option<PrimedWatch<T>>>>,
	notify: Arc<Notify>,
}

impl<T> Default for Watch<T> {
	fn default() -> Self {
		Self::new()
	}
}

impl<T> Watch<T> {
	pub fn new() -> Watch<T> {
		Watch {
			value: Arc::new(RwLock::new(None)),
			notify: Arc::new(Notify::new()),
		}
	}

	pub fn update(&self, new_value: T) {
		let mut inner_watch = self.value.write().expect("lock should not be poisoned");

		if let Some(inner_watch) = inner_watch.as_ref() {
			inner_watch.update(new_value);
		} else {
			*inner_watch = Some(PrimedWatch {
				value: Arc::new(RwLock::new(new_value)),
			});
		}

		self.notify.notify_waiters();
	}

	pub async fn primed(&self) -> PrimedWatch<T> {
		loop {
			let notification = self.notify.notified();

			let inner_watch = self.value.read().expect("lock should not be poisoned");

			if let Some(inner_watch) = inner_watch.as_ref() {
				return PrimedWatch {
					value: inner_watch.value.clone(),
				};
			}

			notification.await;
		}
	}
}

impl<T: Clone> Watch<T> {
	pub fn latest(&self) -> Option<T> {
		let inner_watch = self.value.read().expect("lock should not be poisoned");

		let Some(inner_watch) = inner_watch.as_ref() else {
			return None;
		};

		Some(inner_watch.latest())
	}
}

#[derive(Debug, Clone)]
pub struct PrimedWatch<T> {
	value: Arc<RwLock<T>>,
}

impl<T> PrimedWatch<T> {
	pub fn update(&self, new_value: T) {
		let mut value = self.value.write().expect("lock should not be poisoned");
		*value = new_value;
	}
}

impl<T: Clone> PrimedWatch<T> {
	pub fn latest(&self) -> T {
		let value = self.value.read().expect("lock should not be poisoned");
		value.clone()
	}
}
