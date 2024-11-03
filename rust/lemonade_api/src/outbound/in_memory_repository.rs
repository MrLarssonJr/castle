use crate::domain::lemonade::models::{ApiKeyResource, Filter};
use crate::domain::lemonade::ports::store;
use crate::domain::lemonade::ports::store::errors::{StoreFindApiKeyError, StoreFindOneUserError};
use crate::domain::lemonade::ports::store::{CreateUserOutcome, FindApiKeyFilter, FindUserFilter};
use crate::domain::lemonade::{models::UserActor, ports::store::Store};
use itertools::Itertools;
use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

#[derive(Debug, Default)]
pub struct InMemoryRepository {
	state: Arc<RwLock<State>>,
}

#[derive(Debug, Default)]
struct State {
	api_keys: Vec<ApiKeyResource>,
	users: Vec<UserActor>,
}

impl InMemoryRepository {
	pub fn new() -> InMemoryRepository {
		Self::default()
	}

	fn read(&self) -> RwLockReadGuard<'_, State> {
		self.state.read().expect("lock should not be poisoned")
	}

	fn write(&self) -> RwLockWriteGuard<'_, State> {
		self.state.write().expect("lock should not be poisoned")
	}
}

impl Store for InMemoryRepository {
	async fn create_user(
		&self,
		new_user: &UserActor,
	) -> Result<CreateUserOutcome, store::errors::StoreCreateUserError> {
		let mut state = self.write();

		if state.users.iter().any(|user| user.id == new_user.id) {
			return Ok(CreateUserOutcome::Collision);
		}

		state.users.push(new_user.clone());

		Ok(CreateUserOutcome::Success)
	}

	async fn find_one_api_key(
		&self,
		filter: FindApiKeyFilter<'_>,
	) -> Result<ApiKeyResource, StoreFindApiKeyError> {
		let state = self.read();

		let api_key = state
			.api_keys
			.iter()
			.filter(|resource| filter.matches(resource))
			.exactly_one()
			.cloned()
			.map_err(|err| StoreFindApiKeyError::MoreThanOneFound(err.count()))?;

		Ok(api_key)
	}

	async fn find_one_user(
		&self,
		filter: FindUserFilter<'_>,
	) -> Result<UserActor, StoreFindOneUserError> {
		let state = self.read();

		let user = state
			.users
			.iter()
			.filter(|resource| filter.matches(resource))
			.exactly_one()
			.cloned()
			.map_err(|err| StoreFindOneUserError::MoreThanOneFound(err.count()))?;

		Ok(user)
	}
}
