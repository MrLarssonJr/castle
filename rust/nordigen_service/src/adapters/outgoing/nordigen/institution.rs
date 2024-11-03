use crate::adapters::outgoing::nordigen::NordigenApiInstitutionId;
use iso_country::Country;

pub struct NordigenApiInstitution {
	pub id: NordigenApiInstitutionId,
	pub name: Box<str>,
	pub countries: Vec<Country>,
}

pub struct InstitutionsFilterOptions {}
