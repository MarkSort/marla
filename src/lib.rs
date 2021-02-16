use std::{collections::HashMap, convert::Infallible, net::SocketAddr, panic::AssertUnwindSafe, sync::Arc};

use futures::{future::FutureExt, pin_mut, select};
use hyper::{Method, http::request::Parts, server::conn::AddrStream};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request as HyperRequest, Response, Server, StatusCode};
use regex::Captures;
use tokio::signal::ctrl_c;
use tokio::sync::{Mutex, oneshot::{Receiver, Sender}};
use uuid::Uuid;

use self::config::{MarlaConfig, RegexPath, Route};

pub mod config;

pub async fn serve<B: 'static + Send + Clone> (config: MarlaConfig<B>, bundle: B) {
    let listen_addr = config.listen_addr.clone();

    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel::<()>();
    let shutdown_tx = Arc::new(Mutex::new(Some(shutdown_tx)));

    let make_svc = make_service_fn(move |conn: &AddrStream| {
        let config = config.clone();
        let bundle = bundle.clone();
        let remote_addr = conn.remote_addr();
        let shutdown_tx = shutdown_tx.clone();

        println!("{} | new connection", remote_addr);

        async move {
            Ok::<_, Infallible>(service_fn(move |hyper_request| {
                let config = config.clone();
                let bundle = bundle.clone();
                let shutdown_tx = shutdown_tx.clone();
                async move {
                    match AssertUnwindSafe(
                        handle_request(hyper_request, remote_addr, config, bundle, shutdown_tx)
                    ).catch_unwind().await {
                        Ok(result) => result,
                        Err(_) => Ok(
                            Response::builder()
                                .status(StatusCode::INTERNAL_SERVER_ERROR)
                                .body(Body::from("internal server error\n"))
                                .unwrap()
                        )
                    }
                    
                }
            }))
        }
    });

    let server: Server<_, _> = Server::bind(&listen_addr).serve(make_svc);

    if let Err(e) = server.with_graceful_shutdown(shutdown_signal(shutdown_rx)).await {
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

async fn handle_request<B: 'static + Clone> (
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

    let mut path_captures = None;
    let method_map = match config.static_path_routes.get(path.as_str()) {
        None => {
            match check_regex_routes(path.as_str(), config.regex_path_routes) {
                None => {
                    match if config.router.is_some() {
                        let output = (config.router.unwrap())(path.as_str(), request, body, bundle).await;
                        request = output.0;
                        body = output.1;
                        bundle = output.2;
                        match output.3 {
                            None => None,
                            Some(method_map) => {
                                Some(method_map)
                            }
                        }
                    } else {
                        None
                    } {
                        None => return Ok(Response::builder()
                            .status(StatusCode::NOT_FOUND)
                            .body(Body::from("not found\n"))
                            .unwrap()),
                        Some(method_map) => method_map
                    }
                },
                Some(output) => {
                    path_captures = Some(output.0);
                    output.1
                }
            }
        }
        Some(method_map) => (*method_map).clone(),
    };

    let route = match method_map.get(method) {
        None => {
            return Ok(Response::builder()
                .status(StatusCode::METHOD_NOT_ALLOWED)
                .body(Body::from("method not allowed\n"))
                .unwrap())
        }
        Some(route) => route,
    };

    // moves into RegexRouter eventually
    if let Some(path_captures) = path_captures {
        let mut path_params = vec![];

        for i in 1..path_captures.len() {
            path_params.push(path_captures.get(i).unwrap().as_str().to_string());
        }

        request = Request {
            id,
            head: request.head,
            remote_addr,
            path_params,
            shutdown_tx: request.shutdown_tx,
        };
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
            return Ok(either.right().unwrap())
        }
    }

    Ok((route.handler)(request, body, bundle).await)
}

fn check_regex_routes<B>(path: &str, regex_path_routes: Vec<RegexPath<B>>) -> Option<(Captures, HashMap<Method, Route<B>>)> {
    for regex_path_route in regex_path_routes {
        if let Some(path_params) = regex_path_route.regex.captures(path) {
            return Some((path_params, regex_path_route.routes))
        }
    }

    None
}

pub struct Request {
    pub id: Uuid,
    pub head: Parts,
    pub remote_addr: SocketAddr,
    pub path_params: Vec<String>,
    pub shutdown_tx: Arc<Mutex<Option<Sender<()>>>>,
}
