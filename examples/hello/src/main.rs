use std::net::SocketAddr;

use hyper::{Body, Method, Response};
use macro_rules_attribute::macro_rules_attribute;
use marla::config::{MarlaConfig, Route};
use marla::routing::RegexPath;
use marla::{async_handler, serve, Request};
use regex::Regex;

#[tokio::main]
async fn main() {
    let marla_config = MarlaConfig::builder(SocketAddr::from(([127, 0, 0, 1], 3000)))
        .add_router(Box::new(vec![RegexPath {
            regex: Regex::new("^/hello/([a-zA-Z]{1,30})$").unwrap(),
            routes: vec![(Method::GET, Route::new(hello))].into_iter().collect(),
        }]))
        .build();

    serve(marla_config, ()).await;
}

#[macro_rules_attribute(async_handler!)]
pub async fn hello(request: Request, _body: Option<Body>, _bundle: ()) -> Response<Body> {
    Response::new(Body::from(format!("Hello, {}\n", request.path_params[0])))
}
