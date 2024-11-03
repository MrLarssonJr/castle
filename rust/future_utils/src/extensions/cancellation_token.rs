use std::future::Future;
use tokio_util::sync::CancellationToken;

pub trait CancellationTokenExt {
	fn cancel_when_done<Output>(
		&self,
		fut: impl Future<Output = Output> + Send,
	) -> impl Future<Output = Output> + Send;
}

impl CancellationTokenExt for CancellationToken {
	async fn cancel_when_done<Output>(&self, fut: impl Future<Output = Output> + Send) -> Output {
		let res = fut.await;
		self.cancel();
		res
	}
}

#[cfg(test)]
mod tests {
	use crate::extensions::cancellation_token::CancellationTokenExt;
	use tokio_util::sync::CancellationToken;

	#[tokio::test]
	async fn cancels_token() {
		// Arrange
		let token = CancellationToken::new();
		let future = token.cancel_when_done(async {});

		// Act
		future.await;

		// Assert
		assert!(token.is_cancelled())
	}

	#[tokio::test]
	async fn gives_result_when_nested_in_cancellation() {
		// Arrange
		let token = CancellationToken::new();
		let future = token.run_until_cancelled(token.cancel_when_done(async { 5 }));

		// Act
		let res = future.await;

		// Assert
		assert_eq!(Some(5), res);
	}
}
