use std::{collections::HashMap, net::SocketAddr, time::Duration};

use async_std::task;
use futures::{pin_mut, select, FutureExt};
use hyper::{Body, Client, Method, Response, StatusCode};
use macro_rules_attribute::macro_rules_attribute;

use marla::{
    async_handler,
    config::{MarlaConfig, Route},
    serve, Request,
};

#[async_std::test]
async fn test_catch_undwind_500() {
    let test = (|| async {
        // give server time to start
        task::sleep(Duration::from_millis(100)).await;

        let uri = "http://localhost:3001/throw500".parse().unwrap();
        let mut response = Client::new().get(uri).await.unwrap();

        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);

        let body = hyper::body::to_bytes(response.body_mut()).await.unwrap();
        let body = String::from_utf8(body.to_vec()).unwrap();

        assert_eq!(body, "internal server error\n");
    })()
    .fuse();

    let static_path_router: HashMap<&'static str, HashMap<Method, Route<()>>> = vec![(
        "/throw500",
        vec![(
            Method::GET,
            Route {
                handler: throw500,
                middleware: Some(vec![]),
            },
        )]
        .into_iter()
        .collect(),
    )]
    .into_iter()
    .collect();

    let marla_config = MarlaConfig {
        routers: vec![Box::new(static_path_router)],
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
async fn throw500(_request: Request, _body: Option<Body>, _bundle: ()) -> Response<Body> {
    let x = vec![()];
    let _ = x[1];

    Response::new(Body::empty())
}
