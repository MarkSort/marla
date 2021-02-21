use hyper::{Body, Response, StatusCode};
use macro_rules_attribute::macro_rules_attribute;

use crate::{async_handler, async_handler_generic, Request};

#[macro_rules_attribute(async_handler_generic!)]
pub async fn not_found<B>(_request: Request, _body: Option<Body>, _bundle: B) -> Response<Body> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::from("not found\n"))
        .unwrap()
}

#[macro_rules_attribute(async_handler_generic!)]
pub async fn method_not_allowed<B>(
    _request: Request,
    _body: Option<Body>,
    _bundle: B,
) -> Response<Body> {
    Response::builder()
        .status(StatusCode::METHOD_NOT_ALLOWED)
        .body(Body::from("method not allowed\n"))
        .unwrap()
}

#[macro_rules_attribute(async_handler!)]
pub async fn internal_server_error() -> Response<Body> {
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Body::from("internal server error\n"))
        .unwrap()
}
