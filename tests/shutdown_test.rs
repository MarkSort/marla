use std::{collections::HashMap, net::SocketAddr, time::Duration};

use async_std::task;
use futures::{join, pin_mut, select, FutureExt};
use hyper::{Body, Client, Method, Request as HyperRequest, Response, StatusCode};
use macro_rules_attribute::macro_rules_attribute;

use marla::{
    async_handler,
    config::{MarlaConfig, Route},
    serve, Request,
};

#[async_std::test]
async fn test_shutdown() {
    let timeout = task::sleep(Duration::from_secs(1)).fuse();

    let test = (|| async {
        let static_path_router: HashMap<&'static str, HashMap<Method, Route<()>>> = vec![(
            "/shutdown",
            vec![(Method::POST, Route::new(shutdown))]
                .into_iter()
                .collect(),
        )]
        .into_iter()
        .collect();

        let marla_config = MarlaConfig::builder(SocketAddr::from(([127, 0, 0, 1], 3001)))
            .add_router(Box::new(static_path_router))
            .build();

        let server = serve(marla_config, ());

        join!(
            server,
            (|| async {
                // give server time to start
                task::sleep(Duration::from_millis(100)).await;

                let mut response = Client::new()
                    .request(
                        HyperRequest::builder()
                            .method("POST")
                            .uri("http://localhost:3001/shutdown")
                            .body(Body::empty())
                            .unwrap(),
                    )
                    .await
                    .unwrap();

                assert_eq!(response.status(), StatusCode::OK);

                let body = hyper::body::to_bytes(response.body_mut()).await.unwrap();
                let body = String::from_utf8(body.to_vec()).unwrap();

                assert_eq!(body, "shutdown\n");
            })()
        );
    })()
    .fuse();

    pin_mut!(timeout, test);

    let result = select! {
        _ = timeout => Err(()),
        _ = test => Ok(()),
    };

    assert!(result.is_ok());
}

#[macro_rules_attribute(async_handler!)]
async fn shutdown(request: Request, _body: Option<Body>, _bundle: ()) -> Response<Body> {
    if let Some(tx) = request.shutdown_tx.lock().await.take() {
        match tx.send(()) {
            Ok(_) => Response::new(Body::from("shutdown\n")),
            Err(_) => Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from("error\n"))
                .unwrap(),
        }
    } else {
        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from("error\n"))
            .unwrap()
    }
}
