use crate::domain::lemonade::models::ActorId;
use crate::domain::lemonade::models::{ApiKey, CreateUserOptions, UserActor};
use std::future::Future;
use std::sync::Arc;

pub trait LemonadeService {
	fn authenticate_with_api_key<'l>(
		&self,
		key: &'l ApiKey,
	) -> impl Future<Output = Result<Arc<ActorId>, errors::AuthenticateWithApiKeyError>> + Send;

	fn authenticate_with_password<'l>(
		&self,
		id: &'l str,
		password: &'l str,
	) -> impl Future<Output = Result<Arc<ActorId>, errors::AuthenticateWithApiPasswordError>> + Send;

	fn create_user(
		&self,
		options: CreateUserOptions,
	) -> impl Future<Output = Result<UserActor, errors::CreateUserError>> + Send;

	fn find_user<'l>(
		&self,
		actor: &ActorId,
		user_id: &'l ActorId,
	) -> impl Future<Output = Result<UserActor, errors::FindUserError>> + Send;
}

pub mod errors {
	use crate::domain::lemonade::models::{ActorId, ApiKey};
	use crate::domain::lemonade::ports::store;
	use crate::domain::lemonade::ports::store::errors::StoreFindOneUserError;
	use thiserror::Error;

	#[derive(Debug, Error)]
	#[error("could not authenticate actor with API key {key}")]
	pub enum AuthenticateWithApiKeyError {
		Store {
			key: Box<ApiKey>,
			#[source]
			source: store::errors::StoreFindApiKeyError,
		},
	}

	#[derive(Debug, Error)]
	#[error("could not authenticate actor with password")]
	pub enum AuthenticateWithApiPasswordError {
		Store {
			id: Box<ActorId>,
			#[source]
			source: StoreFindOneUserError,
		},
		BadPassword,
	}

	#[derive(Debug, Error)]
	pub enum CreateUserError {
		#[error("failed to create user in store")]
		Store {
			id: Box<ActorId>,
			#[source]
			source: store::errors::StoreCreateUserError,
		},
	}

	#[derive(Debug, Error)]
	pub enum FindUserError {
		#[error("failed to find user with id {id} in store")]
		Store {
			id: Box<ActorId>,
			#[source]
			source: store::errors::StoreFindOneUserError,
		},
	}
}
