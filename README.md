# Marla - Async Web Server Framework for Rust

Marla is a handler and middleware based web server framework for Rust.

Handlers can be called based on static path maps, regex based paths, and fully custom router functions.

Middleware can be configured to run for all requests by default and overridden for specific routes.

## Production Readiness / Stability

Not yet recommended for production use.  Not guaranteed to ever be ready.

## Example

`Cargo.toml`:
```toml
hyper = { version = "0.14", features = ["full"] }
macro_rules_attribute = "0.0"
marla = "0.1.0-alpha.0"
regex = "1.4"
tokio = { version = "1.0",  features = ["full"] }
```

`main.rs`:
```rust
use std::collections::HashMap;
use std::net::SocketAddr;

use hyper::{Body, Method, Response};
use macro_rules_attribute::macro_rules_attribute;
use marla::{Request, serve, async_handler};
use marla::config::{MarlaConfig, RegexPath, Route};
use regex::Regex;

#[tokio::main]
async fn main() {
    let marla_config = MarlaConfig {

        regex_path_routes: vec![
            RegexPath{
                regex: Regex::new("^/hello/([a-zA-Z]{1,30})$").unwrap(),
                routes: vec![
                    (Method::GET, Route { handler: hello, middleware: None }),
                ].into_iter().collect()
            },
        ],

        static_path_routes: HashMap::new(),
        router: None,
        middleware: vec![],
        listen_addr: SocketAddr::from(([127, 0, 0, 1], 3000)),
    };

    serve(marla_config, ()).await;
}

#[macro_rules_attribute(async_handler!)]
pub async fn hello(
    request: Request,
    _body: Option<Body>,
    _bundle: (),
) -> Response<Body> {
    Response::new(Body::from(format!("Hello, {}\n", request.path_params[0])))
}
```

## Features

- Three ways to route requests to handlers
  - Static Paths
  - Regex Paths - captured patterns are passed to the handler
  - Custom Router Function - for example, check a database for dynamic paths
- Automatic HTTP 404 responses when paths are not found, and HTTP 415 when methods are not supported
- Post-Routing / Pre-Request Middleware
  - You provide a default list of Middleware to run for all requests
  - Override the default Middleware for individual routes
- App defined "Bundle" can be modified by Middleware and is passed to all requests.  Example properties:
  - Database Connection Pools
  - Validated Authentication / Authorization Details
  - Parsed Request Bodies

## Future Enhancements

- Documentation, examples, documentation... docmentation
- Catch unwinding panics and respond with HTTP 500
- Make built-in error responses customizable
- Replace the built in ways to route with implementations of a Router trait
- Replace or re-export http/hyper types, etc.
- Macros for easier to read handler configuration
