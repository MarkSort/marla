use std::{net::SocketAddr, time::Duration};

use async_std::task;
use futures::{pin_mut, select, FutureExt};
use hyper::{Body, Client, Response, StatusCode};
use macro_rules_attribute::macro_rules_attribute;

use marla::{async_handler, config::MarlaConfig, serve, Request};

#[async_std::test]
async fn test_default_not_found_handler() {
    let test = (|| async {
        // give server time to start
        task::sleep(Duration::from_millis(100)).await;

        let uri = "http://localhost:3001/".parse().unwrap();
        let mut response = Client::new().get(uri).await.unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);

        let body = hyper::body::to_bytes(response.body_mut()).await.unwrap();
        let body = String::from_utf8(body.to_vec()).unwrap();

        assert_eq!(body, "not found\n");
    })()
    .fuse();

    let marla_config = MarlaConfig::builder(SocketAddr::from(([127, 0, 0, 1], 3001))).build();

    let server = serve(marla_config, ()).fuse();

    pin_mut!(server, test);

    let result = select! {
        _ = server => Err(()),
        _ = test => Ok(()),
    };

    assert!(result.is_ok());
}

#[async_std::test]
async fn test_custom_not_found_handler() {
    let test = (|| async {
        // give server time to start
        task::sleep(Duration::from_millis(100)).await;

        let uri = "http://localhost:3002/".parse().unwrap();
        let mut response = Client::new().get(uri).await.unwrap();

        assert_eq!(response.status(), StatusCode::IM_A_TEAPOT);

        let body = hyper::body::to_bytes(response.body_mut()).await.unwrap();
        let body = String::from_utf8(body.to_vec()).unwrap();

        assert_eq!(body, "teapot\n");
    })()
    .fuse();

    let marla_config = MarlaConfig::builder(SocketAddr::from(([127, 0, 0, 1], 3002)))
        .set_not_found_handler(teapot)
        .build();

    let server = serve(marla_config, ()).fuse();

    pin_mut!(server, test);

    let result = select! {
        _ = server => Err(()),
        _ = test => Ok(()),
    };

    assert!(result.is_ok());
}

#[macro_rules_attribute(async_handler!)]
pub async fn teapot(_request: Request, _body: Option<Body>, _bundle: ()) -> Response<Body> {
    Response::builder()
        .status(StatusCode::IM_A_TEAPOT)
        .body(Body::from("teapot\n"))
        .unwrap()
}
