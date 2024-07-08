use crate::model;
use async_graphql::futures_util::{Stream, StreamExt, TryStreamExt};
use async_graphql::{Context, Object, Result, SimpleObject};
use mongodb::bson::doc;
use mongodb::options::FindOptions;
use mongodb::{bson, Client};
use std::sync::Arc;
use tokio::join;
use uuid::{NoContext, Timestamp, Uuid};

pub struct UserQuery;

#[Object]
impl UserQuery {
	async fn users<'ctx>(
		&self,
		ctx: &Context<'ctx>,
		first: Option<i64>,
		after: Option<Uuid>,
	) -> Result<UserConnection> {
		let mongo_client = ctx.data::<Client>()?;
		let collection = mongo_client
			.database("lemonade")
			.collection::<model::User>("users");

		let filter = if let Some(after) = after {
			let after = bson::Uuid::from_uuid_1(after);
			doc! { "_id": { "$gt": after } }
		} else {
			doc! {}
		};

		let options = if let Some(first) = first {
			FindOptions::builder().limit(first).build()
		} else {
			FindOptions::builder().build()
		};

		let mut cursor = collection.find(filter, options).await?;
		let mut users = Vec::with_capacity(cursor.size_hint().0);
		let mut ids = None;
		while let Some(elem) = cursor.next().await {
			let elem = elem?;

			ids = Some(match ids {
				Some((first_id, last_id)) => (first_id, elem.id),
				None => (elem.id, elem.id),
			});

			let node = User::from(elem);
			let cursor = node.id;
			let edge = UserEdge { node, cursor };

			users.push(edge);
		}

		let exists = |filter| async {
			Result::<bool, mongodb::error::Error>::Ok(
				collection.find(filter, None).await?.next().await.is_some(),
			)
		};

		let Some((first_id, last_id)) = ids else {
			return Ok(UserConnection {
				edges: vec![],
				page_info: PageInfo {
					has_next_page: false,
					has_previous_page: false,
					start_cursor: None,
					end_cursor: None,
				},
			});
		};

		let (has_before, has_after) = join!(
			exists(doc! { "_id": { "$lt": first_id }}),
			exists(doc! { "_id": { "$gt": last_id }})
		);

		let has_previous_page = has_before?;
		let has_next_page = has_after?;

		let page_info = PageInfo {
			has_next_page,
			has_previous_page,
			start_cursor: Some(first_id.to_uuid_1()),
			end_cursor: Some(last_id.to_uuid_1()),
		};

		let connection = UserConnection {
			edges: users,
			page_info,
		};

		Ok(connection)
	}
}

pub struct UserMutation;

#[Object]
impl UserMutation {
	async fn create_user<'ctx>(&self, ctx: &Context<'ctx>, name: Arc<str>) -> Result<User> {
		let mongo_client = ctx.data::<Client>()?;

		let user = model::User {
			name,
			id: Uuid::new_v7(Timestamp::now(NoContext)).into(),
		};

		mongo_client
			.database("lemonade")
			.collection::<model::User>("users")
			.insert_one(&user, None)
			.await?;

		Ok(user.into())
	}

	async fn delete_user<'ctx>(&self, ctx: &Context<'ctx>, id: Uuid) -> Result<Option<User>> {
		let mongo_client = ctx.data::<Client>()?;

		let _id = bson::Uuid::from_uuid_1(id);
		let filter = doc! {
			"_id": _id
		};

		let res = mongo_client
			.database("lemonade")
			.collection::<model::User>("users")
			.find_one_and_delete(filter, None)
			.await?
			.map(User::from);

		Ok(res)
	}
}

#[derive(SimpleObject)]
struct User {
	id: Uuid,
	name: Arc<str>,
}

impl From<model::User> for User {
	fn from(value: model::User) -> Self {
		User {
			id: value.id.to_uuid_1(),
			name: value.name.clone(),
		}
	}
}

#[derive(SimpleObject)]
#[graphql(rename_fields = "camelCase")]
struct PageInfo {
	has_next_page: bool,
	has_previous_page: bool,
	start_cursor: Option<Uuid>,
	end_cursor: Option<Uuid>,
}

#[derive(SimpleObject)]
#[graphql(rename_fields = "camelCase")]
struct UserEdge {
	node: User,
	cursor: Uuid,
}

#[derive(SimpleObject)]
#[graphql(rename_fields = "camelCase")]
struct UserConnection {
	edges: Vec<UserEdge>,
	page_info: PageInfo,
}
