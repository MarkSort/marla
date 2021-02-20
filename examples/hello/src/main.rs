use std::net::SocketAddr;

use hyper::{Body, Method, Response};
use macro_rules_attribute::macro_rules_attribute;
use marla::config::{MarlaConfig, RegexPath, Route};
use marla::{async_handler, serve, Request};
use regex::Regex;

#[tokio::main]
async fn main() {
    let marla_config = MarlaConfig {
        routers: vec![Box::new(vec![RegexPath {
            regex: Regex::new("^/hello/([a-zA-Z]{1,30})$").unwrap(),
            routes: vec![(
                Method::GET,
                Route {
                    handler: hello,
                    middleware: None,
                },
            )]
            .into_iter()
            .collect(),
        }])],

        middleware: vec![],
        listen_addr: SocketAddr::from(([127, 0, 0, 1], 3000)),
    };

    serve(marla_config, ()).await;
}

#[macro_rules_attribute(async_handler!)]
pub async fn hello(request: Request, _body: Option<Body>, _bundle: ()) -> Response<Body> {
    Response::new(Body::from(format!("Hello, {}\n", request.path_params[0])))
}
