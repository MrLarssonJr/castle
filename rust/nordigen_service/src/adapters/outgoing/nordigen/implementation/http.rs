use crate::adapters::outgoing::nordigen::{
	Expiring, InstitutionsFilterOptions, NordigenApi, NordigenApiAccessToken, NordigenApiAccount,
	NordigenApiAccountId, NordigenApiInstitution, NordigenApiInstitutionId, NordigenApiPage,
	NordigenApiRefreshToken, NordigenApiRequisition, NordigenApiRequisitionCreateOptions,
	NordigenApiRequisitionId, NordigenApiRequisitionStatus, NordigenApiTransaction,
	NordigenApiTransactionId, Secret,
};
use chrono::{NaiveDate, TimeDelta, Utc};
use error::InfallibleResultExt;
use iban::Iban;
use iso_country::Country;
use iso_currency::Currency;
use reqwest::Client;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::marker::PhantomData;
use std::str::FromStr;
use thiserror::Error;
use url::Url;

pub struct HttpNordigenApi<Reference> {
	base: Url,
	client: Client,
	_pd: PhantomData<Reference>,
}

impl<Reference> HttpNordigenApi<Reference> {
	pub fn new(base: Url) -> HttpNordigenApi<Reference> {
		HttpNordigenApi {
			client: Client::new(),
			base,
			_pd: PhantomData::default(),
		}
	}
}

impl<Reference: Send + Sync + FromStr> NordigenApi for HttpNordigenApi<Reference>
where
	<Reference as FromStr>::Err: 'static + Error + Send,
{
	type Reference = Reference;
	type GetAccountError = reqwest::Error;
	type GetAccountTransactionsError = reqwest::Error;
	type GetInstitutionsError = reqwest::Error;
	type GetRequisitionsError = GetRequisitionsError<Reference::Err>;
	type CreateRequisitionError = reqwest::Error;
	type GetRequisitionError = reqwest::Error;
	type DeleteRequisitionError = reqwest::Error;
	type ObtainTokenPairError = reqwest::Error;
	type RefreshAccessTokenError = reqwest::Error;

	async fn get_account(
		&self,
		access_token: &NordigenApiAccessToken,
		id: &NordigenApiAccountId,
	) -> Result<NordigenApiAccount, Self::GetAccountError> {
		let url = self
			.base
			.join("accounts/")
			.and_then(|url| url.join(id.as_ref()))
			.and_then(|url| url.join("/"))
			.expect("to be valid url");

		#[derive(Deserialize)]
		struct ResponseBody {
			iban: Iban,
		}

		let response = self
			.client
			.get(url)
			.bearer_auth(access_token.as_ref())
			.send()
			.await?;
		let response_body = response.json::<ResponseBody>().await?;

		let account = NordigenApiAccount {
			iban: response_body.iban,
		};

		Ok(account)
	}

	async fn get_account_transactions(
		&self,
		access_token: &NordigenApiAccessToken,
		id: &NordigenApiAccountId,
		from: Option<NaiveDate>,
		to: Option<NaiveDate>,
	) -> Result<Vec<NordigenApiTransaction>, Self::GetAccountTransactionsError> {
		let mut url = self
			.base
			.join("accounts/")
			.and_then(|url| url.join(id.as_ref()))
			.and_then(|url| url.join("/transactions/"))
			.expect("to be valid url");

		{
			let mut query_pairs = url.query_pairs_mut();

			if let Some(from) = from {
				let from = from.to_string();
				query_pairs.append_pair("date_from", &from);
			}

			if let Some(to) = to {
				let to = to.to_string();
				query_pairs.append_pair("date_to", &to);
			}
		}

		#[derive(Deserialize)]
		struct ResponseBody {
			transactions: ResponseBodyTransactions,
		}

		#[derive(Deserialize)]
		struct ResponseBodyTransactions {
			booked: Vec<ResponseBodyBookedTransaction>,
		}

		#[derive(Deserialize)]
		struct ResponseBodyBookedTransaction {
			#[serde(rename = "internalTransactionId")]
			internal_transaction_id: Box<str>,
			#[serde(rename = "transactionAmount")]
			transaction_amount: ResponseBodyBookedTransactionAmount,
			#[serde(rename = "bookingDate")]
			booking_date: NaiveDate,
			#[serde(alias = "creditorAccount")]
			#[serde(alias = "debtorAccount")]
			reciprocal_account: ResponseBodyBookedTransactionReciprocalAccount,
		}
		#[derive(Deserialize)]
		struct ResponseBodyBookedTransactionAmount {
			currency: Currency,
			amount: Decimal,
		}
		#[derive(Deserialize)]
		struct ResponseBodyBookedTransactionReciprocalAccount {
			iban: Iban,
		}

		let response = self
			.client
			.get(url)
			.bearer_auth(access_token.as_ref())
			.send()
			.await?;
		let response_body = response.json::<ResponseBody>().await?;

		let transactions = response_body
			.transactions
			.booked
			.into_iter()
			.map(|transaction| {
				let id = NordigenApiTransactionId::from_str(transaction.internal_transaction_id)
					.unwrap_infallible();

				NordigenApiTransaction {
					id,
					reciprocal_account: transaction.reciprocal_account.iban,
					amount: transaction.transaction_amount.amount,
					currency: transaction.transaction_amount.currency,
					booking_date: transaction.booking_date,
				}
			})
			.collect();

		Ok(transactions)
	}

	async fn get_institutions(
		&self,
		access_token: &NordigenApiAccessToken,
		_options: InstitutionsFilterOptions,
	) -> Result<Vec<NordigenApiInstitution>, Self::GetInstitutionsError> {
		let url = self.base.join("institutions/").expect("to be valid url");

		type ResponseBody = Vec<ResponseBodyInstitution>;

		#[derive(Deserialize)]
		struct ResponseBodyInstitution {
			id: Box<str>,
			name: Box<str>,
			countries: Vec<Country>,
		}

		let response = self
			.client
			.get(url)
			.bearer_auth(access_token.as_ref())
			.send()
			.await?;
		let response_body = response.json::<ResponseBody>().await?;

		let institutions = response_body
			.into_iter()
			.map(|institution| {
				let id = NordigenApiInstitutionId::from_str(institution.id).unwrap_infallible();

				NordigenApiInstitution {
					id,
					name: institution.name,
					countries: institution.countries,
				}
			})
			.collect();

		Ok(institutions)
	}

	async fn get_requisitions(
		&self,
		access_token: &NordigenApiAccessToken,
		limit: Option<usize>,
		offset: Option<usize>,
	) -> Result<NordigenApiPage<NordigenApiRequisition<Reference>>, Self::GetRequisitionsError> {
		let mut url = self.base.join("requisitions/").expect("to be valid url");

		{
			let mut query_pairs = url.query_pairs_mut();

			if let Some(limit) = limit {
				let limit = limit.to_string();
				query_pairs.append_pair("limit", &limit);
			}

			if let Some(offset) = offset {
				let offset = offset.to_string();
				query_pairs.append_pair("offset", &offset);
			}
		}

		#[derive(Deserialize)]
		struct ResponseBody {
			results: Vec<ResponseBodyRequisition>,
		}

		#[derive(Deserialize)]
		struct ResponseBodyRequisition {
			id: Box<str>,
			status: Status,
			institution_id: Box<str>,
			accounts: Vec<Box<str>>,
			reference: Option<Box<str>>,
			link: Url,
		}

		#[derive(Deserialize)]
		enum Status {
			#[serde(rename = "CR")]
			Created,
			#[serde(rename = "GC")]
			GivingConsent,
			#[serde(rename = "UA")]
			UndergoingAuthentication,
			#[serde(rename = "RJ")]
			Rejected,
			#[serde(rename = "SA")]
			SelectingAccounts,
			#[serde(rename = "GA")]
			GrantingAccess,
			#[serde(rename = "LN")]
			Linked,
			#[serde(rename = "EX")]
			Expired,
		}

		let response = self
			.client
			.get(url)
			.bearer_auth(access_token.as_ref())
			.send()
			.await?;
		let response_body = response.json::<ResponseBody>().await?;

		let requisitions: Result<Vec<_>, _> = response_body
			.results
			.into_iter()
			.map(|requisition| {
				let id = NordigenApiRequisitionId::from_str(requisition.id).unwrap_infallible();
				let status = match requisition.status {
					Status::Created => NordigenApiRequisitionStatus::Created,
					Status::GivingConsent => NordigenApiRequisitionStatus::GivingConsent,
					Status::UndergoingAuthentication => {
						NordigenApiRequisitionStatus::UndergoingAuthentication
					}
					Status::Rejected => NordigenApiRequisitionStatus::Rejected,
					Status::SelectingAccounts => NordigenApiRequisitionStatus::SelectingAccounts,
					Status::GrantingAccess => NordigenApiRequisitionStatus::GrantingAccess,
					Status::Linked => NordigenApiRequisitionStatus::Linked,
					Status::Expired => NordigenApiRequisitionStatus::Expired,
				};
				let institution_id = NordigenApiInstitutionId::from_str(requisition.institution_id)
					.unwrap_infallible();
				let reference = requisition
					.reference
					.as_deref()
					.map(Reference::from_str)
					.transpose()
					.map_err(GetRequisitionsError::ReferenceParseError)?;
				let link = requisition.link;
				let accounts = requisition
					.accounts
					.into_iter()
					.map(|account_id| {
						NordigenApiAccountId::from_str(account_id).unwrap_infallible()
					})
					.collect();

				Ok::<_, GetRequisitionsError<Reference::Err>>(NordigenApiRequisition {
					id,
					status,
					institution_id,
					reference,
					accounts,
					link,
				})
			})
			.collect();

		let requisitions = requisitions?;

		let page = NordigenApiPage {
			count: requisitions.len(),
			results: requisitions,
		};

		Ok(page)
	}

	async fn create_requisition<'i, 'r>(
		&self,
		access_token: &NordigenApiAccessToken,
		requisition: NordigenApiRequisitionCreateOptions<'i, 'r>,
	) -> Result<NordigenApiRequisition<Reference>, Self::CreateRequisitionError> {
		todo!()
	}

	async fn get_requisition(
		&self,
		access_token: &NordigenApiAccessToken,
		id: &NordigenApiRequisitionId,
	) -> Result<NordigenApiRequisition<Reference>, Self::GetRequisitionError> {
		todo!()
	}

	async fn delete_requisition(
		&self,
		access_token: &NordigenApiAccessToken,
		id: &NordigenApiRequisitionId,
	) -> Result<(), Self::DeleteRequisitionError> {
		todo!()
	}

	async fn obtain_token_pair(
		&self,
		secret: &Secret,
	) -> Result<
		(
			Expiring<NordigenApiAccessToken>,
			Expiring<NordigenApiRefreshToken>,
		),
		Self::ObtainTokenPairError,
	> {
		let url = self.base.join("token/new/").expect("to be valid URL");

		#[derive(Serialize)]
		struct RequestBody<'l> {
			secret_id: &'l str,
			secret_key: &'l str,
		}

		#[derive(Deserialize)]
		struct ResponseBody {
			access: Box<str>,
			access_expires: i64,
			refresh: Box<str>,
			refresh_expires: i64,
		}

		let body = RequestBody {
			secret_id: secret.id.as_ref(),
			secret_key: secret.key.as_ref(),
		};

		let start = Utc::now();

		let response = self.client.post(url).json(&body).send().await?;
		let response_body = response.json::<ResponseBody>().await?;

		let access_token =
			NordigenApiAccessToken::from_str(response_body.access).unwrap_infallible();
		let refresh_token =
			NordigenApiRefreshToken::from_str(response_body.refresh).unwrap_infallible();

		let access_token_expires_at = start + TimeDelta::seconds(response_body.access_expires);
		let refresh_token_expires_at = start + TimeDelta::seconds(response_body.refresh_expires);

		let access_token = Expiring {
			value: access_token,
			expires_at: access_token_expires_at,
		};

		let refresh_token = Expiring {
			value: refresh_token,
			expires_at: refresh_token_expires_at,
		};

		Ok((access_token, refresh_token))
	}

	async fn refresh_access_token(
		&self,
		refresh_token: &NordigenApiRefreshToken,
	) -> Result<Expiring<NordigenApiAccessToken>, Self::RefreshAccessTokenError> {
		let url = self.base.join("token/refresh/").expect("to be valid URL");

		#[derive(Serialize)]
		struct RequestBody<'l> {
			refresh: &'l str,
		}

		#[derive(Deserialize)]
		struct ResponseBody {
			access: Box<str>,
			access_expires: i64,
		}

		let body = RequestBody {
			refresh: refresh_token.as_ref(),
		};

		let start = Utc::now();

		let response = self.client.post(url).json(&body).send().await?;
		let response_body = response.json::<ResponseBody>().await?;

		let access_token =
			NordigenApiAccessToken::from_str(response_body.access).unwrap_infallible();

		let access_token_expires_at = start + TimeDelta::seconds(response_body.access_expires);

		let access_token = Expiring {
			value: access_token,
			expires_at: access_token_expires_at,
		};

		Ok(access_token)
	}
}

#[derive(Debug, Error)]
pub enum GetRequisitionsError<E: Error> {
	#[error("an http error occurred")]
	Http(#[from] reqwest::Error),
	#[error("could not parse requisitions reference")]
	ReferenceParseError(#[source] E),
}
