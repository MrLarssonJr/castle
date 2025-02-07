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

use snafu::{ResultExt, Whatever};
use sqlx::pool::PoolConnection;
use sqlx::postgres::PgPoolOptions;
use sqlx::{PgConnection, PgPool, Postgres};
use std::ops::{Deref, DerefMut};

mod tables;
pub(crate) mod types;

#[derive(Debug, Clone)]
pub struct Database {
	connection_pool: PgPool,
}

impl Database {
	pub async fn new(url: &str) -> Result<Self, Whatever> {
		let connection_pool = PgPoolOptions::new()
			.connect(url)
			.await
			.with_whatever_context(|_| "unable to create database, unable to connect")?;

		Ok(Database { connection_pool })
	}

	pub async fn conn(&self) -> Result<Connection<'static>, Whatever> {
		let conn = self
			.connection_pool
			.acquire()
			.await
			.with_whatever_context(|_| "")?;

		Ok(Connection(InnerConnection::Pooled(conn)))
	}

	pub async fn begin_transaction(&self) -> Result<Transaction, Whatever> {
		let tx = self
			.connection_pool
			.begin()
			.await
			.with_whatever_context(|_| "")?;

		Ok(Transaction(tx))
	}
}

pub struct Transaction(sqlx::Transaction<'static, Postgres>);

impl Transaction {
	pub fn conn(&mut self) -> Connection {
		Connection(InnerConnection::Normal(self.0.deref_mut()))
	}

	pub async fn commit_transaction(self) -> Result<(), Whatever> {
		self.0.commit().await.with_whatever_context(|_| "")
	}
}

pub struct Connection<'l>(InnerConnection<'l>);

impl<'l> Deref for Connection<'l> {
	type Target = PgConnection;

	fn deref(&self) -> &Self::Target {
		self.0.deref()
	}
}

impl<'l> DerefMut for Connection<'l> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.0.deref_mut()
	}
}
enum InnerConnection<'l> {
	Pooled(PoolConnection<Postgres>),
	Normal(&'l mut PgConnection),
}

impl<'l> Deref for InnerConnection<'l> {
	type Target = PgConnection;
	fn deref(&self) -> &Self::Target {
		match self {
			InnerConnection::Pooled(conn) => conn.deref(),
			InnerConnection::Normal(conn) => &*conn,
		}
	}
}

impl<'l> DerefMut for InnerConnection<'l> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		match self {
			InnerConnection::Pooled(conn) => conn.deref_mut(),
			InnerConnection::Normal(conn) => &mut *conn,
		}
	}
}
