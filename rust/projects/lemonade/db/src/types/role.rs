#[derive(Debug, Copy, Clone, sqlx::Type)]
#[sqlx(type_name = "role")]
#[sqlx(rename_all = "lowercase")]
pub(crate) enum Role {
	Admin,
}

impl From<Role> for lemonade_model::Role {
	fn from(value: Role) -> Self {
		match value {
			Role::Admin => lemonade_model::Role::Admin,
		}
	}
}

impl From<lemonade_model::Role> for Role {
	fn from(value: lemonade_model::Role) -> Self {
		match value {
			lemonade_model::Role::Admin => Role::Admin,
		}
	}
}
