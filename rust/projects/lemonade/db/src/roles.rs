use crate::types::role::Role;
use lemonade_model::Role as ModelRole;
use snafu::{ResultExt, Whatever};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn has_role(db: &PgPool, user_id: Uuid, role: ModelRole) -> Result<bool, Whatever> {
	let role = Role::from(role);

	let (has_role,) =
		sqlx::query_as("SELECT EXISTS(SELECT * FROM roles WHERE user_id = $1 AND role = $2);")
			.bind(user_id)
			.bind(role)
			.fetch_one(db)
			.await
			.with_whatever_context(|_| {
				format!("failed to check if user with id {user_id} had role {role:?}")
			})?;

	Ok(has_role)
}
