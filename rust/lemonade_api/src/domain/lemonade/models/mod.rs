use crate::domain::lemonade::ports::random_provider::RandomProvider;
use crate::macros::new_str;
use error::InfallibleResultExt;
use std::any::type_name;
use std::error::Error;
use std::sync::Arc;
use thiserror::Error;

pub trait Filter {
	type Resource;
	fn matches(&self, value: &Self::Resource) -> bool;
}

new_str!(pub ActorId);

#[derive(Debug, Clone)]
pub struct ApiKeyResource {
	pub actor_id: Arc<ActorId>,
	pub api_key: Arc<ApiKey>,
}

impl ActorId {
	pub fn generate_random<RP: RandomProvider>(random_provider: &RP) -> Box<ActorId> {
		let random_value = random_provider.random();
		let random_value = format!("{random_value:032X}");
		let random_value = random_value
			.as_str()
			.parse::<&ActorId>()
			.unwrap_infallible();
		let random_value = Box::from(random_value);
		random_value
	}
}

new_str!(pub ApiKey);

pub struct CreateUserOptions<'l> {
	pub name: &'l str,
}

#[derive(Debug, Clone)]
pub struct Actor;

#[derive(Debug, Clone)]
pub struct UserActor {
	pub id: Arc<ActorId>,
	pub name: Arc<str>,
}

#[derive(Debug, Error)]
#[error("domain dependency {port} returned an error unknown to the domain")]
pub struct DomainDependencyError {
	port: &'static str,
	#[source]
	error: Box<dyn Error + Send>,
}

impl DomainDependencyError {
	pub fn new<Port>(error: impl Error + Send + 'static) -> DomainDependencyError {
		DomainDependencyError {
			port: type_name::<Port>(),
			error: Box::new(error),
		}
	}
}
