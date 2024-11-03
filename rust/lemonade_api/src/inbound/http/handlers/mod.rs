use crate::domain::lemonade::ports::lemonade_service::LemonadeService;
use crate::inbound::http::app_state::AppState;
use axum::Router;

pub fn build_router<LS: 'static + Send + Sync + LemonadeService>() -> Router<AppState<LS>> {
	Router::new()
		.nest("/me", me::build_router())
		.nest("/users", users::build_router())
}

mod me;

mod users {
	use crate::domain::lemonade::ports::lemonade_service::LemonadeService;
	use crate::inbound::http::app_state::AppState;
	use axum::routing::post;
	use axum::Router;

	pub fn build_router<LS: 'static + Send + Sync + LemonadeService>() -> Router<AppState<LS>> {
		Router::new()
			.nest("/:user_id", user_id::build_router())
			.route("/", post(post::handler))
	}

	mod post {
		use crate::domain::lemonade::models::CreateUserOptions;
		use crate::domain::lemonade::ports::lemonade_service::LemonadeService;
		use crate::inbound::http::error::AppError;
		use axum::extract::{FromRequest, FromRequestParts, State};
		use axum::response::IntoResponse;
		use axum::Json;
		use axum_extra::extract::JsonDeserializer;
		use std::future::Future;
		use std::sync::Arc;

		mod dto {
			pub use request::*;
			mod request {
				use crate::domain::lemonade::models::CreateUserOptions;
				use serde::Deserialize;
				use std::borrow::Cow;

				#[derive(Debug, Deserialize)]
				pub struct Request<'l> {
					#[serde(borrow)]
					name: Cow<'l, str>,
				}

				impl<'l> From<&'l Request<'l>> for CreateUserOptions<'l> {
					fn from(request: &'l Request<'l>) -> CreateUserOptions<'l> {
						let name: &'l str = &request.name;

						CreateUserOptions { name }
					}
				}
			}

			pub use response::*;

			mod response {
				use crate::domain::lemonade::models::UserActor;
				use serde::Serialize;
				use std::sync::Arc;

				#[derive(Debug, Serialize)]
				pub struct Response {
					id: Arc<str>,
					name: Arc<str>,
				}

				impl From<UserActor> for Response {
					fn from(user: UserActor) -> Self {
						Response {
							id: user.id.into_str_arc(),
							name: user.name,
						}
					}
				}
			}
		}

		pub async fn handler<LS: LemonadeService + Send>(
			State(lemonade_service): State<Arc<LS>>,
			request_body: JsonDeserializer<dto::Request<'_>>,
		) -> Result<Json<dto::Response>, AppError> {
			let request_body = request_body.deserialize()?;
			let create_user_options = CreateUserOptions::from(&request_body);

			let response_body = lemonade_service
				.create_user(create_user_options)
				.await?
				.into();

			Ok(Json(response_body))
		}
	}

	mod get {}

	mod user_id {
		use crate::domain::lemonade::ports::lemonade_service::LemonadeService;
		use crate::inbound::http::app_state::AppState;
		use axum::routing::get;
		use axum::Router;

		pub fn build_router<LS: 'static + Send + Sync + LemonadeService>() -> Router<AppState<LS>> {
			Router::new().route("/", get(get::handler))
		}

		mod user_id_path_param {
			use crate::domain::lemonade::models::ActorId;
			use error::InfallibleResultExt;
			use serde::Deserialize;

			#[derive(Debug, Deserialize)]
			pub struct UserIdPathParam {
				user_id: Box<str>,
			}

			impl From<UserIdPathParam> for Box<ActorId> {
				fn from(value: UserIdPathParam) -> Self {
					value.user_id.parse::<&ActorId>().unwrap_infallible().into()
				}
			}
		}

		mod get {
			use crate::domain::lemonade::models::ActorId;
			use crate::domain::lemonade::ports::lemonade_service::LemonadeService;
			use crate::inbound::http::error::AppError;
			use crate::inbound::http::extractors::Authentication;
			use crate::inbound::http::handlers::users::user_id::user_id_path_param::UserIdPathParam;
			use axum::extract::{Path, State};
			use axum::Json;
			use std::sync::Arc;

			mod dto {
				pub use response::*;
				mod response {
					use crate::domain::lemonade::models::UserActor;
					use serde::Serialize;
					use std::sync::Arc;

					#[derive(Debug, Serialize)]
					pub struct Response {
						id: Arc<str>,
						name: Arc<str>,
					}

					impl From<UserActor> for Response {
						fn from(user: UserActor) -> Self {
							Response {
								id: user.id.into_str_arc(),
								name: user.name,
							}
						}
					}
				}
			}

			pub async fn handler<LS: LemonadeService + Send>(
				authentication: Authentication,
				State(lemonade_service): State<Arc<LS>>,
				Path(user_id): Path<UserIdPathParam>,
			) -> Result<Json<dto::Response>, AppError> {
				let actor = authentication.actor_id();
				let user_id: Box<ActorId> = user_id.into();

				let user = lemonade_service.find_user(actor, &user_id).await?.into();

				Ok(Json(user))
			}
		}

		mod delete {}

		mod sessions {
			mod post {
				mod dto {
					mod request {
						use serde::Deserialize;
						use std::borrow::Cow;

						#[derive(Debug, Deserialize)]
						pub struct Request<'l> {
							#[serde(borrow)]
							pub password: Cow<'l, str>,
						}
					}

					mod response {
						use serde::Serialize;

						#[derive(Debug, Serialize)]
						pub struct Response {}
					}
				}

				// pub async fn handler<LS: LemonadeService + Send>(
				// 	State(lemonade_service): State<Arc<LS>>,
				// 	Path(user_id): Path<UserIdPathParam>,
				// 	request_body: JsonDeserializer<dto::Request<'_>>,
				// ) -> Result<Json<dto::Response>, ApiError> {
				// 	todo!()
				// }
			}

			mod id {
				mod delete {}
			}
		}
	}
}
