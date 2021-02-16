use std::{collections::HashMap};

use async_trait::async_trait;
use hyper::{Body, Method};

use crate::{Request, config::{RegexPath, Route}};

#[async_trait]
pub trait Router<B>: RouterClone<B> + Send {
    async fn route(&self, path: &str, request: Request, body: Option<Body>, bundle: B) ->
        (Request, Option<Body>, B, Option<HashMap<Method, Route<B>>>);
}

pub trait RouterClone<B> {
    fn clone_box(&self) -> Box<dyn Router<B>>;
}

impl<B, T> RouterClone<B> for T
where
    T: 'static + Router<B> + Clone
{
    fn clone_box(&self) -> Box<dyn Router<B>> {
        Box::new(self.clone())
    }
}

impl<B> Clone for Box<dyn Router<B>> {
    fn clone(&self) -> Box<dyn Router<B>> {
        self.clone_box()
    }
}

#[async_trait]
impl<B: Clone + Send + 'static> Router<B> for HashMap<&'static str, HashMap<Method, Route<B>>> {
    async fn route(&self, path: &str, request: Request, body: Option<Body>, bundle: B) ->
        (Request, Option<Body>, B, Option<HashMap<Method, Route<B>>>)
    {
        let methods = match self.get(path) {
            None => None,
            Some(methods) => Some(methods.clone())
        };
        (request, body, bundle, methods)
        
    }
}

#[async_trait]
impl<B: Clone + Send + 'static> Router<B> for Vec<RegexPath<B>> {
    async fn route(&self, path: &str, request: Request, body: Option<Body>, bundle: B) ->
        (Request, Option<Body>, B, Option<HashMap<Method, Route<B>>>)
    {
        for regex_path_route in self {
            if let Some(path_captures) = regex_path_route.regex.captures(path) {
                let mut path_params = vec![];

                for i in 1..path_captures.len() {
                    path_params.push(path_captures.get(i).unwrap().as_str().to_string());
                }
        
                let request = Request {
                    id: request.id,
                    head: request.head,
                    remote_addr: request.remote_addr,
                    path_params,
                    shutdown_tx: request.shutdown_tx,
                };
        
                let methods = Some(regex_path_route.routes.clone());
                return (request, body, bundle, methods)
            }
        }
        
        (request, body, bundle, None)
    }
}
