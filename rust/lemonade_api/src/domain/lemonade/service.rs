use crate::domain::lemonade::ports::lemonade_service::errors::{
	AuthenticateWithApiKeyError, AuthenticateWithApiPasswordError, CreateUserError, FindUserError,
};
use crate::domain::lemonade::ports::store::{CreateUserOutcome, FindApiKeyFilter, FindUserFilter};
use crate::domain::lemonade::{
	models::{ActorId, ApiKey, CreateUserOptions, UserActor},
	ports::{lemonade_service::LemonadeService, random_provider::RandomProvider, store::Store},
};
use std::sync::Arc;

pub struct Service<RP, LR> {
	lemonade_repository: LR,
	random_provider: RP,
}

impl<RP, LR> Service<RP, LR> {
	pub fn new(random_provider: RP, lemonade_repository: LR) -> Service<RP, LR> {
		Service {
			lemonade_repository,
			random_provider,
		}
	}
}

impl<RP: RandomProvider + Sync, LR: Store + Sync> LemonadeService for Service<RP, LR> {
	async fn authenticate_with_api_key(
		&self,
		key: &ApiKey,
	) -> Result<Arc<ActorId>, AuthenticateWithApiKeyError> {
		let filter = FindApiKeyFilter {
			api_key: Some(key),
			..Default::default()
		};

		let resource = self
			.lemonade_repository
			.find_one_api_key(filter)
			.await
			.map_err(|source| AuthenticateWithApiKeyError::Store {
				key: Box::from(key),
				source,
			})?;

		Ok(resource.actor_id)
	}

	async fn authenticate_with_password<'l>(
		&self,
		_id: &'l str,
		_password: &'l str,
	) -> Result<Arc<ActorId>, AuthenticateWithApiPasswordError> {
		todo!()
	}

	async fn create_user(
		&self,
		options: CreateUserOptions<'_>,
	) -> Result<UserActor, CreateUserError> {
		loop {
			let id = Arc::<ActorId>::from(ActorId::generate_random(&self.random_provider));
			let name = options.name.into();

			let user = UserActor {
				id: id.clone(),
				name,
			};

			let outcome = self
				.lemonade_repository
				.create_user(&user)
				.await
				.map_err(|source| CreateUserError::Store {
					id: Box::from(id.as_ref()),
					source,
				})?;

			match outcome {
				CreateUserOutcome::Success => break Ok(user),
				CreateUserOutcome::Collision => {}
			}
		}
	}

	async fn find_user<'l>(
		&self,
		_actor: &ActorId,
		user_id: &'l ActorId,
	) -> Result<UserActor, FindUserError> {
		// TODO: add authorization
		let filter = FindUserFilter {
			id: Some(user_id),
			..Default::default()
		};

		let resource = self
			.lemonade_repository
			.find_one_user(filter)
			.await
			.map_err(|source| FindUserError::Store {
				id: Box::from(user_id),
				source,
			})?;

		Ok(resource)
	}
}
