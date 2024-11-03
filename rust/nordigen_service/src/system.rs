use crate::adapters::outgoing::nordigen::{
	Expiring, InstitutionsFilterOptions, NordigenApi, NordigenApiAccessToken, NordigenApiAccountId
	,
};
use crate::nordigen_service::NordigenService;
use async_stream::try_stream;
use chrono::NaiveDate;
use constrained_str::{constrained_str, Validator};
use error::InfallibleResultExt;
use futures::Stream;
use iban::Iban;
use iso_country::Country;
use iso_currency::Currency;
use rust_decimal::Decimal;
use std::convert::Infallible;
use std::fmt::Debug;
use std::sync::Arc;
use sync_utils::PrimedWatch;

pub struct BankingService<NordigenApi> {
	nordigen_service: Arc<NordigenService<NordigenApi>>,
}

impl<N: NordigenApi<Reference = ClientId> + Debug + Send + Sync + 'static + Clone>
	BankingService<N>
{
	pub fn new(
		nordigen_api: N,
		access_token: PrimedWatch<Expiring<NordigenApiAccessToken>>,
	) -> BankingService<N> {
		let nordigen_service = NordigenService::new(nordigen_api, access_token).into();

		BankingService { nordigen_service }
	}

	pub fn get_institutions(&self) -> impl '_ + Stream<Item = Result<Institution, ()>> {
		try_stream! {
			let institutions = self.nordigen_service.get_institutions(InstitutionsFilterOptions {});

			for await institution in institutions {
				let institution = institution.map_err(|_| ())?;
				let institution = Institution {
					name: institution.name,
					countries: institution.countries,
					id: InstitutionId::from_str(institution.id).unwrap_infallible(),
				};

				yield institution;
			}
		}
	}

	// pub async fn create_link(client: Client, institution: In) -> Result<Link, ()> {}
	//
	// pub async fn delete_link(&self, client: Client, link_id: LinkId) -> Result<Link, ()> {}
	//
	// pub async fn list_links(&self, client: Client) -> impl Stream<Item = Result<Link, ()>> {
	// 	todo!()
	// }

	fn get_accounts<'l>(
		&'l self,
		client_id: &'l ClientId,
	) -> impl 'l + Stream<Item = Result<(NordigenApiAccountId, Iban), ()>> {
		try_stream! {
			let requisitions = self.nordigen_service.get_requisitions();

			for await requisition in requisitions {
				let requisition = requisition.map_err(|_| ())?;

				if !requisition.reference.as_ref().is_some_and(|reference| reference == client_id) {
					continue;
				}

				for account_id in requisition.accounts {
					let account = self.nordigen_service.get_account(&account_id).await;
					let account = account.map_err(|_| ())?;

					yield (account_id, account.iban);
				}
			}
		}
	}

	pub async fn get_transactions<'l>(
		&'l self,
		client_id: &'l ClientId,
	) -> impl 'l + Stream<Item = Result<Transaction, ()>> {
		try_stream! {
			let accounts = self.get_accounts(client_id);

			for await account in accounts {
				let (account_id, iban) = account.map_err(|_| ())?;

				let transactions = self.nordigen_service.get_account_transactions(&account_id, None, None).await;
				let transactions = transactions.map_err(|_| ())?;

				for transaction in transactions {
					yield Transaction {
						account: iban,
						reciprocal_account: transaction.reciprocal_account,
						amount: transaction.amount,
						currency: transaction.currency,
						id: TransactionId::from_str(transaction.id).unwrap_infallible(),
						date: transaction.booking_date,
					}
				}
			}
		}
	}
}

pub struct Institution {
	name: Box<str>,
	countries: Vec<Country>,
	id: InstitutionId,
}

pub struct Link {
	id: LinkId,
	account_ids: Vec<AccountId>,
}

pub struct Transaction {
	account: Iban,
	reciprocal_account: Iban,
	amount: Decimal,
	currency: Currency,
	id: TransactionId,
	date: NaiveDate,
}

pub struct AccountIdValidator;

impl Validator for AccountIdValidator {
	type Error = Infallible;

	fn validate(v: &str) -> Result<&str, Self::Error> {
		// every str is valid account id
		Ok(v)
	}
}

constrained_str!(AccountId, AccountIdValidator);

pub struct TransactionIdValidator;

impl Validator for TransactionIdValidator {
	type Error = Infallible;

	fn validate(v: &str) -> Result<&str, Self::Error> {
		// every str is valid account id
		Ok(v)
	}
}

constrained_str!(TransactionId, TransactionIdValidator);

pub struct LinkIdValidator;

impl Validator for LinkIdValidator {
	type Error = Infallible;

	fn validate(v: &str) -> Result<&str, Self::Error> {
		// every str is valid account id
		Ok(v)
	}
}

constrained_str!(LinkId, LinkIdValidator);

pub struct ClientIdValidator;

impl Validator for ClientIdValidator {
	type Error = Infallible;

	fn validate(v: &str) -> Result<&str, Self::Error> {
		// every str is valid account id
		Ok(v)
	}
}

constrained_str!(ClientId, ClientIdValidator);

pub struct InstitutionIdValidator;

impl Validator for InstitutionIdValidator {
	type Error = Infallible;

	fn validate(v: &str) -> Result<&str, Self::Error> {
		// every str is valid account id
		Ok(v)
	}
}

constrained_str!(InstitutionId, InstitutionIdValidator);
