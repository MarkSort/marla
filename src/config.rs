use std::{collections::HashMap, future::Future, net::SocketAddr, pin::Pin};

use either::Either;
use hyper::{Body, Method, Response};
use regex::Regex;

use crate::{Request, routing::Router};

#[derive(Clone)]
pub struct Route<B> {
    pub handler: fn(Request, Option<Body>, B) -> Pin<Box<dyn Future<Output = Response<Body>> + Send >>,
    pub middleware:  Option<Vec< fn(Request, Option<Body>, B) -> Pin<Box<dyn Future<Output= Either<(Request, Option<Body>, B), Response<Body>> > + Send >> >>,
}

#[derive(Clone)]
pub struct RegexPath<B> {
    pub regex: Regex,
    pub routes: HashMap<Method, Route<B>>,
}

#[derive(Clone)]
pub struct MarlaConfig<B: 'static> {
    pub routers: Vec<Box<dyn Router<B>>>,
    pub middleware: Vec< fn(Request, Option<Body>, B) -> Pin<Box<dyn Future<Output= Either<(Request, Option<Body>, B), Response<Body>> > + Send >> >,
    pub listen_addr: SocketAddr,
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
