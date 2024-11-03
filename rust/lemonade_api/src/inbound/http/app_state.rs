use axum::extract::FromRef;
use std::ops::Deref;
use std::sync::Arc;

#[derive(Debug)]
pub struct AppState<LS> {
	lemonade_service: Arc<LS>,
}

impl<LS> AppState<LS> {
	pub fn lemonade_service(&self) -> &LS {
		self.lemonade_service.deref()
	}
}

// manual impl of Clone due to #[derive] using incorrect bounds
// https://github.com/rust-lang/rust/issues/26925
impl<LS> Clone for AppState<LS> {
	fn clone(&self) -> Self {
		AppState {
			lemonade_service: self.lemonade_service.clone(),
		}
	}
}

impl<LS> AppState<LS> {
	pub fn new(lemonade_service: LS) -> AppState<LS> {
		AppState {
			lemonade_service: Arc::new(lemonade_service),
		}
	}
}

impl<LS> FromRef<AppState<LS>> for Arc<LS> {
	fn from_ref(app_state: &AppState<LS>) -> Self {
		app_state.lemonade_service.clone()
	}
}
