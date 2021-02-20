use std::{net::SocketAddr, time::Duration};

use async_std::task;
use futures::{pin_mut, select, FutureExt};
use hyper::{Body, Client, Method, Response, StatusCode};
use macro_rules_attribute::macro_rules_attribute;

use marla::{
    async_handler,
    config::{MarlaConfig, Route},
    routing::RegexPath,
    serve, Request,
};
use regex::Regex;

#[async_std::test]
async fn test_regex_route() {
    let test = (|| async {
        // give server time to start
        task::sleep(Duration::from_millis(100)).await;

        let uri = "http://localhost:3001/hello".parse().unwrap();
        let mut response = Client::new().get(uri).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = hyper::body::to_bytes(response.body_mut()).await.unwrap();
        let body = String::from_utf8(body.to_vec()).unwrap();

        assert_eq!(body, "hello\n");

        let uri = "http://localhost:3001/bye".parse().unwrap();
        let response = Client::new().get(uri).await.unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    })()
    .fuse();

    let marla_config = MarlaConfig {
        routers: vec![Box::new(vec![RegexPath {
            regex: Regex::new("^/([a-zA-Z]{5})$").unwrap(),
            routes: vec![(
                Method::GET,
                Route {
                    handler: five_letters,
                    middleware: None,
                },
            )]
            .into_iter()
            .collect(),
        }])],
        middleware: vec![],
        listen_addr: SocketAddr::from(([127, 0, 0, 1], 3001)),
    };

    let server = serve(marla_config, ()).fuse();

    pin_mut!(server, test);

    let result = select! {
        _ = server => Err(()),
        _ = test => Ok(()),
    };

    assert!(result.is_ok());
}

#[macro_rules_attribute(async_handler!)]
async fn five_letters(request: Request, _body: Option<Body>, _bundle: ()) -> Response<Body> {
    Response::new(Body::from(format!("{}\n", request.path_params[0])))
}
