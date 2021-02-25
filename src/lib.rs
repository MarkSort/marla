use std::{
    collections::HashMap, convert::Infallible, net::SocketAddr, panic::AssertUnwindSafe, sync::Arc,
};

use futures::{future::FutureExt, pin_mut, select};
use hyper::service::{make_service_fn, service_fn};
use hyper::{http::request::Parts, server::conn::AddrStream, Method};
use hyper::{Body, Request as HyperRequest, Response, Server};
use tokio::signal::ctrl_c;
use tokio::sync::{
    oneshot::{Receiver, Sender},
    Mutex,
};
use uuid::Uuid;

use self::config::{MarlaConfig, Route};

pub mod config;
pub mod provided;
pub mod routing;

pub async fn serve<B: 'static + Send + Clone>(config: MarlaConfig<B>, bundle: B) {
    let listen_addr = config.listen_addr.clone();

    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    let shutdown_tx = Arc::new(Mutex::new(Some(shutdown_tx)));

    let make_svc = make_service_fn(|conn: &AddrStream| {
        let config = config.clone();
        let bundle = bundle.clone();
        let remote_addr = conn.remote_addr();
        let shutdown_tx = shutdown_tx.clone();

        println!("{} | new connection", remote_addr);

        async move {
            Ok::<_, Infallible>(service_fn(move |hyper_request| {
                let internal_server_error_handler = config.internal_server_error_handler.clone();

                let config = config.clone();
                let bundle = bundle.clone();
                let shutdown_tx = shutdown_tx.clone();
                async move {
                    match AssertUnwindSafe(handle_request(
                        hyper_request,
                        remote_addr,
                        config,
                        bundle,
                        shutdown_tx,
                    ))
                    .catch_unwind()
                    .await
                    {
                        Ok(result) => result,
                        Err(_) => Ok((internal_server_error_handler)().await),
                    }
                }
            }))
        }
    });

    let server: Server<_, _> = Server::bind(&listen_addr).serve(make_svc);

    if let Err(e) = server
        .with_graceful_shutdown(shutdown_signal(shutdown_rx))
        .await
    {
        eprintln!("server error: {}", e);
    }
}

async fn shutdown_signal(shutdown_rx: Receiver<()>) {
    let ctrl_c_fut = ctrl_c().fuse();
    let shutdown_rx_fut = shutdown_rx.fuse();

    pin_mut!(ctrl_c_fut, shutdown_rx_fut);

    let initiator = select! {
        _ = ctrl_c_fut => "ctrl-c",
        _ = shutdown_rx_fut => "shutdown channel",
    };

    println!("graceful shutdown initiated by {}", initiator);
}

async fn handle_request<B: 'static + Clone>(
    hyper_request: HyperRequest<Body>,
    remote_addr: SocketAddr,
    config: MarlaConfig<B>,
    bundle: B,
    shutdown_tx: Arc<Mutex<Option<Sender<()>>>>,
) -> Result<Response<Body>, Infallible> {
    let (head, body) = hyper_request.into_parts();
    let mut body = Some(body);

    let id = Uuid::new_v4();
    let path = head.uri.path().to_string();
    let method = &head.method.clone();

    println!(
        "{} | {} | new request - {} {}",
        remote_addr, id, method, path
    );

    let mut bundle = bundle;
    let mut request = Request {
        id,
        head,
        remote_addr,
        path_params: vec![],
        shutdown_tx,
    };

    let mut merged_method_map: Option<HashMap<Method, Route<B>>> = None;
    for router in config.routers {
        let router_future = router.route(&path, request, body, bundle);
        let output = router_future.await;
        request = output.0;
        body = output.1;
        bundle = output.2;

        if let Some(method_map) = output.3 {
            if let Some(ref mut merged) = merged_method_map {
                for (method, route) in method_map {
                    if !merged.contains_key(&method) {
                        merged.insert(method, route);
                    }
                }
            } else {
                merged_method_map = Some(method_map);
            }
        }
    }
    let method_map = match merged_method_map {
        Some(method_map) => method_map,
        None => return Ok((config.not_found_handler)(request, body, bundle).await),
    };

    let route = match method_map.get(method) {
        None => return Ok((config.method_not_allowed_handler)(request, body, bundle).await),
        Some(route) => route,
    };

    let middleware_vec = if route.middleware.is_some() {
        route.middleware.clone().unwrap()
    } else {
        config.middleware
    };

    for middleware in middleware_vec {
        let either = middleware(request, body, bundle).await;
        if either.is_left() {
            let output = either.left().unwrap();
            request = output.0;
            body = output.1;
            bundle = output.2;
        } else {
            return Ok(either.right().unwrap());
        }
    }

    Ok((route.handler)(request, body, bundle).await)
}

pub struct Request {
    pub id: Uuid,
    pub head: Parts,
    pub remote_addr: SocketAddr,
    pub path_params: Vec<String>,
    pub shutdown_tx: Arc<Mutex<Option<Sender<()>>>>,
}
