mod account;
mod ids;
mod implementation;
mod institution;
mod page;
mod requisition;
mod secret;
mod token;
mod transaction;

pub use account::*;
pub use ids::*;
pub use institution::*;
pub use page::*;
pub use requisition::*;
pub use secret::*;
pub use token::*;
pub use transaction::*;

use chrono::NaiveDate;
use std::error::Error;
use std::future::Future;

pub trait NordigenApi {
	type Reference;
	type GetAccountError: 'static + Error + Send;
	type GetAccountTransactionsError: 'static + Error + Send;
	type GetInstitutionsError: 'static + Error + Send;
	type GetRequisitionsError: 'static + Error + Send;
	type CreateRequisitionError: 'static + Error + Send;
	type GetRequisitionError: 'static + Error + Send;
	type DeleteRequisitionError: 'static + Error + Send;
	type ObtainTokenPairError: 'static + Error + Send;
	type RefreshAccessTokenError: 'static + Error + Send;

	fn get_account(
		&self,
		access_token: &NordigenApiAccessToken,
		id: &NordigenApiAccountId,
	) -> impl Future<Output = Result<NordigenApiAccount, Self::GetAccountError>>;

	fn get_account_transactions(
		&self,
		access_token: &NordigenApiAccessToken,
		id: &NordigenApiAccountId,
		from: Option<NaiveDate>,
		to: Option<NaiveDate>,
	) -> impl Future<Output = Result<Vec<NordigenApiTransaction>, Self::GetAccountTransactionsError>>
	       + Send;

	fn get_institutions(
		&self,
		access_token: &NordigenApiAccessToken,
		options: InstitutionsFilterOptions,
	) -> impl Future<Output = Result<Vec<NordigenApiInstitution>, Self::GetInstitutionsError>> + Send;

	fn get_requisitions(
		&self,
		access_token: &NordigenApiAccessToken,
		limit: Option<usize>,
		offset: Option<usize>,
	) -> impl Future<
		Output = Result<
			NordigenApiPage<NordigenApiRequisition<Self::Reference>>,
			Self::GetRequisitionsError,
		>,
	> + Send;
	fn create_requisition(
		&self,
		access_token: &NordigenApiAccessToken,
		requisition: NordigenApiRequisitionCreateOptions,
	) -> impl Future<
		Output = Result<NordigenApiRequisition<Self::Reference>, Self::CreateRequisitionError>,
	> + Send;
	fn get_requisition(
		&self,
		access_token: &NordigenApiAccessToken,
		id: &NordigenApiRequisitionId,
	) -> impl Future<
		Output = Result<NordigenApiRequisition<Self::Reference>, Self::GetRequisitionError>,
	> + Send;
	fn delete_requisition(
		&self,
		access_token: &NordigenApiAccessToken,
		id: &NordigenApiRequisitionId,
	) -> impl Future<Output = Result<(), Self::DeleteRequisitionError>> + Send;

	fn obtain_token_pair(
		&self,
		secret: &Secret,
	) -> impl Future<
		Output = Result<
			(
				Expiring<NordigenApiAccessToken>,
				Expiring<NordigenApiRefreshToken>,
			),
			Self::ObtainTokenPairError,
		>,
	> + Send;
	fn refresh_access_token(
		&self,
		refresh_token: &NordigenApiRefreshToken,
	) -> impl Future<Output = Result<Expiring<NordigenApiAccessToken>, Self::RefreshAccessTokenError>>
	       + Send;
}
