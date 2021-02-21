use std::{future::Future, net::SocketAddr, pin::Pin};

use either::Either;
use hyper::{Body, Response};

use crate::{provided::error_handlers, routing::Router, Request};

type RouteHandler<B> =
    fn(Request, Option<Body>, B) -> Pin<Box<dyn Future<Output = Response<Body>> + Send>>;

type Middleware<B> = fn(
    Request,
    Option<Body>,
    B,
) -> Pin<
    Box<dyn Future<Output = Either<(Request, Option<Body>, B), Response<Body>>> + Send>,
>;

#[derive(Clone)]
pub struct Route<B> {
    pub handler: RouteHandler<B>,
    pub middleware: Option<Vec<Middleware<B>>>,
}

impl<B> Route<B> {
    pub fn new(handler: RouteHandler<B>) -> Route<B> {
        Route {
            handler,
            middleware: None,
        }
    }
}

pub struct RouteBuilder<B> {
    pub handler: RouteHandler<B>,
    pub middleware: Option<Vec<Middleware<B>>>,
}

#[derive(Clone)]
pub struct MarlaConfig<B: 'static> {
    pub routers: Vec<Box<dyn Router<B>>>,
    pub middleware: Vec<Middleware<B>>,
    pub listen_addr: SocketAddr,
    pub not_found_handler: RouteHandler<B>,
    pub method_not_allowed_handler: RouteHandler<B>,
    pub internal_server_error_handler: fn() -> Pin<Box<dyn Future<Output = Response<Body>> + Send>>,
}

impl<B> MarlaConfig<B> {
    pub fn builder(listen_addr: SocketAddr) -> MarlaConfigBuilder<B> {
        MarlaConfigBuilder {
            routers: vec![],
            middleware: vec![],
            listen_addr,
            not_found_handler: None,
            method_not_allowed_handler: None,
            internal_server_error_handler: None,
        }
    }
}

pub struct MarlaConfigBuilder<B> {
    routers: Vec<Box<dyn Router<B>>>,
    middleware: Vec<Middleware<B>>,
    listen_addr: SocketAddr,
    not_found_handler: Option<RouteHandler<B>>,
    method_not_allowed_handler: Option<RouteHandler<B>>,
    internal_server_error_handler:
        Option<fn() -> Pin<Box<dyn Future<Output = Response<Body>> + Send>>>,
}

impl<B> MarlaConfigBuilder<B> {
    pub fn add_router(mut self, router: Box<dyn Router<B>>) -> MarlaConfigBuilder<B> {
        self.routers.push(router);
        self
    }

    pub fn add_middleware(mut self, middleware: Middleware<B>) -> MarlaConfigBuilder<B> {
        self.middleware.push(middleware);
        self
    }

    pub fn set_not_found_handler(mut self, handler: RouteHandler<B>) -> MarlaConfigBuilder<B> {
        self.not_found_handler = Some(handler);
        self
    }

    pub fn set_method_not_allowed_handler(
        mut self,
        handler: RouteHandler<B>,
    ) -> MarlaConfigBuilder<B> {
        self.method_not_allowed_handler = Some(handler);
        self
    }

    pub fn set_internal_server_error_handler(
        mut self,
        handler: fn() -> Pin<Box<dyn Future<Output = Response<Body>> + Send>>,
    ) -> MarlaConfigBuilder<B> {
        self.internal_server_error_handler = Some(handler);
        self
    }

    pub fn build(self) -> MarlaConfig<B> {
        MarlaConfig {
            routers: self.routers,
            middleware: self.middleware,
            listen_addr: self.listen_addr,
            not_found_handler: if let Some(handler) = self.not_found_handler {
                handler
            } else {
                error_handlers::not_found
            },
            method_not_allowed_handler: if let Some(handler) = self.not_found_handler {
                handler
            } else {
                error_handlers::method_not_allowed
            },
            internal_server_error_handler: if let Some(handler) = self.internal_server_error_handler
            {
                handler
            } else {
                error_handlers::internal_server_error
            },
        }
    }
}

#[macro_export]
macro_rules! async_handler {(
    $( #[$attr:meta] )* // includes doc strings
    $pub:vis
    async
    fn $fname:ident ( $($args:tt)* ) $(-> $Ret:ty)?
    {
        $($body:tt)*
    }
) => (
    $( #[$attr] )*
    #[allow(unused_parens)]
    $pub
    fn $fname ( $($args)* ) -> ::std::pin::Pin<::std::boxed::Box<
        dyn ::std::future::Future<Output = ($($Ret)?)>
            + ::std::marker::Send
    >>
    {
        ::std::boxed::Box::pin(async move { $($body)* })
    }
)}

#[macro_export]
macro_rules! async_handler_generic {(
    $( #[$attr:meta] )* // includes doc strings
    $pub:vis
    async
    fn $fname:ident<$($types:ident)*> ( $($args:tt)* ) $(-> $Ret:ty)?
    {
        $($body:tt)*
    }
) => (
    $( #[$attr] )*
    #[allow(unused_parens)]
    $pub
    fn $fname<$($types)*> ( $($args)* ) -> ::std::pin::Pin<::std::boxed::Box<
        dyn ::std::future::Future<Output = ($($Ret)?)>
            + ::std::marker::Send
    >>
    {
        ::std::boxed::Box::pin(async move { $($body)* })
    }
)}

#[macro_export]
macro_rules! async_router {(
    $( #[$attr:meta] )* // includes doc strings
    $pub:vis
    async
    fn $fname:ident ( $($args:tt)* ) $(-> $Ret:ty)?
    {
        $($body:tt)*
    }
) => (
    $( #[$attr] )*
    #[allow(unused_parens)]
    $pub
    fn $fname ( $($args)* ) -> ::std::pin::Pin<::std::boxed::Box<
        dyn ::std::future::Future<Output = ($($Ret)?)>
            + ::std::marker::Send + '_
    >>
    {
        ::std::boxed::Box::pin(async move { $($body)* })
    }
)}
