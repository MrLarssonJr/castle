use crate::adapters::outgoing::nordigen::ids::{NordigenApiInstitutionId, NordigenApiReference};
use crate::adapters::outgoing::nordigen::{NordigenApiAccountId, NordigenApiRequisitionId};
use url::Url;

pub struct NordigenApiRequisition<Reference> {
	pub id: NordigenApiRequisitionId,
	pub status: NordigenApiRequisitionStatus,
	pub institution_id: NordigenApiInstitutionId,
	pub reference: Option<Reference>,
	pub accounts: Vec<NordigenApiAccountId>,
	pub link: Url,
}

pub enum NordigenApiRequisitionStatus {
	Created,
	GivingConsent,
	UndergoingAuthentication,
	Rejected,
	SelectingAccounts,
	GrantingAccess,
	Linked,
	Expired,
}

pub struct NordigenApiRequisitionCreateOptions<'i, 'r> {
	pub redirect: Url,
	pub institution_id: &'i NordigenApiInstitutionId,
	pub reference: &'r NordigenApiReference,
}
