use std::sync::Arc;

use async_graphql::{Context, Object, Result, SimpleObject};
use serde::{Deserialize, Serialize};

use crate::nordigen_token_client::NordigenTokenClient;
use crate::token_manager::TokenManager;

pub struct InstitutionQuery;

#[Object]
impl InstitutionQuery {
	async fn institutions<'ctx>(&self, ctx: &Context<'ctx>) -> Result<Vec<Institution>> {
		let token_manager = ctx.data::<TokenManager<NordigenTokenClient>>()?;
		let nordigen_client = ctx.data::<http_api::nordigen::Client>()?;

		let access_token = token_manager.access_token().await?;
		let institutions = nordigen_client.institutions(access_token).await?;
		let institutions = institutions.into_iter().map(Institution::from).collect();

		Ok(institutions)
	}
}

#[derive(SimpleObject, Serialize, Deserialize)]
struct Institution {
	pub id: Arc<str>,
	pub name: Arc<str>,
}

impl From<http_api::nordigen::InstitutionsResponse> for Institution {
	fn from(value: http_api::nordigen::InstitutionsResponse) -> Self {
		Institution {
			id: value.id,
			name: value.name,
		}
	}
}
