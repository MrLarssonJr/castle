use std::future::Future;
use std::time::Duration;
use tokio::time::Instant;

const BACKOFF_BASE: Duration = Duration::from_millis(100);
const BACKOFF_LIMIT: Duration = Duration::from_secs(60);
const BACKOFF_SCALING_FACTOR: u32 = 2;
const BACKOFF_RESET_AFTER: Duration = Duration::from_secs(5 * 60);

pub async fn restart_on_failure_with_backoff<
	Output,
	Error,
	F: Future<Output = Result<Output, Error>>,
>(
	mut fut_constructor: impl FnMut() -> F,
	mut on_error: impl FnMut(Duration, Error),
) -> Output {
	let mut backoff = BACKOFF_BASE;

	loop {
		let start = Instant::now();
		let res = fut_constructor().await;
		let time_elapsed = start.elapsed();

		let error = match res {
			Ok(res) => return res,
			Err(error) => error,
		};

		if time_elapsed < BACKOFF_RESET_AFTER {
			backoff *= BACKOFF_SCALING_FACTOR;
			backoff = backoff.min(BACKOFF_LIMIT);
		} else {
			backoff = BACKOFF_BASE;
		}

		on_error(backoff, error);

		tokio::time::sleep(backoff).await;
	}
}

#[cfg(test)]
mod tests {
	use crate::restart::restart_on_failure_with_backoff;
	use std::cell::RefCell;
	use std::time::Duration;
	use tokio::time::Instant;

	#[tokio::test(start_paused = true)]
	async fn backoff() {
		// Arrange
		let test_start = Instant::now();

		let count = RefCell::new(0);
		let task = || async {
			*count.borrow_mut() += 1;

			if *count.borrow() <= 12 {
				return Err(*count.borrow());
			}

			tokio::time::sleep(Duration::from_secs(10 * 60)).await;

			if *count.borrow() <= 24 {
				return Err(*count.borrow());
			}

			Ok(*count.borrow())
		};

		// Act
		let mut errors = Vec::with_capacity(3);

		let start = Instant::now();
		let res = restart_on_failure_with_backoff(task, |backoff, err| {
			println!(
				"[{:6.1}s: ERROR] {err}, will restart in {}s",
				test_start.elapsed().as_secs_f64(),
				backoff.as_secs_f64()
			);
			errors.push(err)
		})
		.await;

		println!("[{:6.1}s:  INFO] {res}", test_start.elapsed().as_secs_f64(),);
		let elapsed_time = start.elapsed();

		// Assert
		assert_eq!(errors, (1..=24).collect::<Vec<_>>());
		assert_eq!(res, 25);
		let expected_cumulative_time_slept = Duration::from_millis(8083 * 1000 + 400);
		assert_eq!(elapsed_time, expected_cumulative_time_slept);
	}
}
