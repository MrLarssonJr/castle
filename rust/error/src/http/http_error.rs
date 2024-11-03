pub trait HttpError<Body> {
	fn into_response(self) -> http::Response<Body>;
}
