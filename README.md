# Marla - Async Web Server Framework for Rust

Marla is a handler and middleware based web server framework for Rust.

Handlers can be called based on static path maps, regex based paths, and fully custom router functions.

Middleware can be configured to run for all requests by default and overridden for specific routes.

## Example

You can run this example in [examples/hello](examples/hello) and browse to http://localhost:3000/hello/world , or use `curl`:

```bash
curl -sS http://localhost:3000/hello/world -D-
```

`Cargo.toml`:
```toml
[dependencies]
hyper = { version = "0.14", features = ["full"] }
macro_rules_attribute = "0.0"
marla = "/path/to/marla" # with `main` branch checked out
regex = "1.4"
tokio = { version = "1.0",  features = ["full"] }
```

`main.rs`:
```rust
use std::net::SocketAddr;

use hyper::{Body, Method, Response};
use macro_rules_attribute::macro_rules_attribute;
use marla::{Request, serve, async_handler};
use marla::config::{MarlaConfig, RegexPath, Route};
use regex::Regex;

#[tokio::main]
async fn main() {
    let marla_config = MarlaConfig {
        routers: vec![Box::new(vec![
            RegexPath{ regex: Regex::new("^/hello/([a-zA-Z]{1,30})$").unwrap(), routes: vec![
                (Method::GET, Route { handler: hello, middleware: None }),
            ].into_iter().collect()},
        ])],

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
- Automatic HTTP 404 responses when paths are not found, and HTTP 405 when methods are not supported
- Panics (unwinding) in handlers or middleware will return HTTP 500 responses
- Post-Routing / Pre-Handler Middleware
  - You provide a default list of Middleware to run for all requests
  - Override the default Middleware for individual routes
  - Middleware can send custom responses, preventing call to handlers
- App defined "Bundle" can be modified by Middleware and is passed to all requests.  Example properties:
  - Database Connection Pools
  - Validated Authentication / Authorization Details
  - Parsed Request Bodies
- Handlers and middleware can initiate graceful server shutdown

## Future Enhancements

- Documentation, examples, documentation... documentation
- More Tests
- Configurable Logging
- More Middleware types
  - Post-Connection / Pre-Request-Received
  - Post-Request-Received / Pre-Routing
  - Post-Handler / Pre-Response-Sent
- Make built-in error responses customizable
- Replace or re-export http/hyper types, etc.
- Macros for easier to read handler configuration
