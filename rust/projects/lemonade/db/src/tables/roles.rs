use crate::types::role::Role;
use crate::Connection;
use lemonade_model::Role as ModelRole;
use snafu::{ResultExt, Whatever};
use std::ops::DerefMut;
use uuid::Uuid;

impl<'c> Connection<'c> {
	pub async fn has_role(&mut self, user_id: Uuid, role: ModelRole) -> Result<bool, Whatever> {
		let role = Role::from(role);

		let (has_role,) =
			sqlx::query_as("SELECT EXISTS(SELECT * FROM roles WHERE user_id = $1 AND role = $2);")
				.bind(user_id)
				.bind(role)
				.fetch_one(self.deref_mut())
				.await
				.with_whatever_context(|_| {
					format!("failed to check if user with id {user_id} had role {role:?}")
				})?;

		Ok(has_role)
	}
}
