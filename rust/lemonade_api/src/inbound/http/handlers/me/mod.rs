use crate::domain::lemonade::ports::lemonade_service::LemonadeService;
use crate::inbound::http::app_state::AppState;
use axum::routing::get;
use axum::Router;

pub fn build_router<LS: 'static + Send + Sync + LemonadeService>() -> Router<AppState<LS>> {
	Router::new().route("/", get(get::handler))
}

mod get;
