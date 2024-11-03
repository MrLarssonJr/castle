use crate::domain::lemonade::models::{ActorId, ApiKey, ApiKeyResource, Filter, UserActor};
use std::future::Future;

pub trait Store {
	fn find_one_api_key(
		&self,
		filter: FindApiKeyFilter,
	) -> impl Future<Output = Result<ApiKeyResource, errors::StoreFindApiKeyError>> + Send;

	fn create_user(
		&self,
		user: &UserActor,
	) -> impl Future<Output = Result<CreateUserOutcome, errors::StoreCreateUserError>> + Send;

	fn find_one_user(
		&self,
		filter: FindUserFilter,
	) -> impl Future<Output = Result<UserActor, errors::StoreFindOneUserError>> + Send;
}

pub enum CreateUserOutcome {
	Collision,
	Success,
}

#[derive(Debug, Default)]
pub struct FindApiKeyFilter<'l> {
	pub actor_id: Option<&'l ActorId>,
	pub api_key: Option<&'l ApiKey>,
}

impl Filter for FindApiKeyFilter<'_> {
	type Resource = ApiKeyResource;

	fn matches(&self, value: &Self::Resource) -> bool {
		match *self {
			FindApiKeyFilter {
				actor_id: Some(actor_id),
				..
			} if actor_id != value.actor_id.as_ref() => false,

			FindApiKeyFilter {
				api_key: Some(api_key),
				..
			} if api_key != value.api_key.as_ref() => false,

			_ => true,
		}
	}
}

#[derive(Debug, Default)]
pub struct FindUserFilter<'l> {
	pub id: Option<&'l ActorId>,
}

impl Filter for FindUserFilter<'_> {
	type Resource = UserActor;

	fn matches(&self, value: &Self::Resource) -> bool {
		match *self {
			FindUserFilter { id: Some(id), .. } if id != value.id.as_ref() => false,
			_ => true,
		}
	}
}

pub mod errors {
	use crate::domain::lemonade::models::DomainDependencyError;
	use thiserror::Error;

	#[derive(Debug, Error)]
	pub enum StoreFindApiKeyError {
		#[error("more than one user found")]
		MoreThanOneFound(usize),
		#[error("API key not found")]
		NotFound,
		#[error("an unknown repository error occurred")]
		Unknown(#[from] DomainDependencyError),
	}

	#[derive(Debug, Error)]
	pub enum StoreCreateUserError {
		#[error("could not create user, an unknown error occurred")]
		Unknown(#[from] DomainDependencyError),
	}

	#[derive(Debug, Error)]
	pub enum StoreFindOneUserError {
		#[error("more than one user found")]
		MoreThanOneFound(usize),
		#[error("user not found")]
		NotFound,
		#[error("an unknown repository error occurred")]
		Unknown(#[from] DomainDependencyError),
	}
}
