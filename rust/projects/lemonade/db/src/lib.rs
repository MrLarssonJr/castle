//! The purpose of gather all SQL into one crate is to simplify
//! schema updates.
//!
//! Initially there's supposed to one module herewithin for each table in the schema,
//! and each module should contain the statements related to that table. Of course, this
//! will break down when there multi-table statements become required. But problem for then…
//!
//! This simplifies schema updates because if a table schema changes, then one will only have
//! to refactor into the relevant modules herewithin. Any consequent internal API changes should
//! (hopefully) be caught by the compiler.

pub mod nordigen_tokens;
pub mod roles;
pub mod sessions;
pub mod users;

pub(crate) mod types {
	pub(crate) mod role {
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
	}
}
