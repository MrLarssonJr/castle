use error::ErrorExt;

mod domain;
mod inbound;
mod macros;
mod outbound;

#[tokio::main]
async fn main() {
	tracing_subscriber::FmtSubscriber::builder()
		.with_max_level(tracing::Level::TRACE)
		.init();

	tracing::info!("starting server");
	let Err(error) = inbound::http::start().await else {
		return;
	};
	tracing::error!("{}", error.to_pretty_string())
}
