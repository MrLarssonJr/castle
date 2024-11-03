use crate::adapters::outgoing::nordigen::{
	Expiring, InstitutionsFilterOptions, NordigenApi, NordigenApiAccessToken, NordigenApiAccount,
	NordigenApiAccountId, NordigenApiInstitution, NordigenApiRequisition,
	NordigenApiRequisitionCreateOptions, NordigenApiRequisitionId, NordigenApiTransaction,
};
use async_stream::try_stream;
use chrono::{DateTime, NaiveDate, Utc};
use error::HttpError;
use futures::Stream;
use http::Response;
use std::fmt::Debug;
use sync_utils::PrimedWatch;
use thiserror::Error;

pub struct NordigenService<N> {
	nordigen_api: N,
	access_token: PrimedWatch<Expiring<NordigenApiAccessToken>>,
}

impl<N: NordigenApi + Debug + Send + Sync + 'static + Clone> NordigenService<N> {
	pub fn new(
		nordigen_api: N,
		access_token: PrimedWatch<Expiring<NordigenApiAccessToken>>,
	) -> NordigenService<N> {
		NordigenService {
			nordigen_api,
			access_token,
		}
	}

	fn access_token<T>(&self) -> Result<NordigenApiAccessToken, NordigenServiceError<T>> {
		let access_token = self.access_token.latest();
		let now = Utc::now();

		if access_token.expires_at < now {
			return Err(NordigenServiceError::ExpiredAccessToken {
				expired_at: access_token.expires_at,
				now,
			});
		}

		Ok(access_token.value)
	}

	pub async fn get_account(
		&self,
		id: &NordigenApiAccountId,
	) -> Result<NordigenApiAccount, NordigenServiceError<N::GetAccountError>> {
		let access_token = self.access_token()?;
		Ok(self.nordigen_api.get_account(&access_token, id).await?)
	}

	pub async fn get_account_transactions(
		&self,
		id: &NordigenApiAccountId,
		from: Option<NaiveDate>,
		to: Option<NaiveDate>,
	) -> Result<Vec<NordigenApiTransaction>, NordigenServiceError<N::GetAccountTransactionsError>>
	{
		let access_token = self.access_token()?;
		Ok(self
			.nordigen_api
			.get_account_transactions(&access_token, id, from, to)
			.await?)
	}

	pub fn get_institutions(
		&self,
		options: InstitutionsFilterOptions,
	) -> impl '_
	       + Stream<
		Item = Result<NordigenApiInstitution, NordigenServiceError<N::GetInstitutionsError>>,
	> {
		try_stream! {
			let access_token = self.access_token()?;

			let institutions = self.nordigen_api
					.get_institutions(&access_token, options)
					.await?;

			for institution in institutions {
				yield institution;
			}
		}
	}

	pub fn get_requisitions(
		&self,
	) -> impl '_
	       + Stream<
		Item = Result<
			NordigenApiRequisition<N::Reference>,
			NordigenServiceError<N::GetRequisitionsError>,
		>,
	> {
		try_stream! {
			let limit = 100;
			let mut offset = 0;

			loop {
				let access_token = self.access_token()?;
				let page = self
					.nordigen_api
					.get_requisitions(&access_token, Some(100), Some(offset))
					.await?;

				if page.results.is_empty() {
					break;
				}

				for requisition in page.results {
					yield requisition;
				}

				offset += limit;
			}
		}
	}

	pub async fn create_requisition<'i, 'r>(
		&self,
		requisition: NordigenApiRequisitionCreateOptions<'i, 'r>,
	) -> Result<NordigenApiRequisition<N::Reference>, NordigenServiceError<N::CreateRequisitionError>>
	{
		let access_token = self.access_token()?;
		Ok(self
			.nordigen_api
			.create_requisition(&access_token, requisition)
			.await?)
	}

	pub async fn get_requisition(
		&self,
		id: &NordigenApiRequisitionId,
	) -> Result<NordigenApiRequisition<N::Reference>, NordigenServiceError<N::GetRequisitionError>>
	{
		let access_token = self.access_token()?;
		Ok(self.nordigen_api.get_requisition(&access_token, id).await?)
	}

	pub async fn delete_requisition(
		&self,
		id: &NordigenApiRequisitionId,
	) -> Result<(), NordigenServiceError<N::DeleteRequisitionError>> {
		let access_token = self.access_token()?;
		Ok(self
			.nordigen_api
			.delete_requisition(&access_token, id)
			.await?)
	}
}

#[derive(Debug, Error)]
pub enum NordigenServiceError<E> {
	#[error(transparent)]
	Api(#[from] E),
	#[error("could not perform Nordigen API request due to access token being expired (expired at {expired_at}, request attempt at {now})")]
	ExpiredAccessToken {
		now: DateTime<Utc>,
		expired_at: DateTime<Utc>,
	},
}

impl<E: HttpError<Body>, Body: From<&'static str>> HttpError<Body> for NordigenServiceError<E> {
	fn into_response(self) -> http::response::Response<Body> {
		match self {
			NordigenServiceError::Api(api_error) => api_error.into_response(),
			NordigenServiceError::ExpiredAccessToken { .. } => {
				let mut response = Response::new("an internal server occurred".into());
				*response.status_mut() = http::StatusCode::INTERNAL_SERVER_ERROR;

				response
			}
		}
	}
}
