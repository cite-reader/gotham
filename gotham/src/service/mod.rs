//! Defines the `GothamService` type which is used to wrap a Gotham application and interface with
//! Tower.

use std::net::SocketAddr;
use std::panic::AssertUnwindSafe;
use std::pin::Pin;
use std::sync::Arc;
use std::thread;

use failure;

use futures::prelude::*;
use futures::task::{self, Poll};
use http::request;
use hyper::service::Service;
use hyper::{Body, Request, Response};
use log::debug;

use crate::handler::NewHandler;

use crate::helpers::http::request::path::RequestPathSegments;
use crate::state::client_addr::put_client_addr;
use crate::state::{set_request_id, State};

mod trap;

/// Wraps a [`NewHandler`] in a Tower [`Service`].
///
/// [`NewHandler`]: ../handler/trait.NewHandler.html
/// [`Service`]: ../../tower_service/trait.Service.html
pub struct GothamService<T>
where
    T: NewHandler + 'static,
{
    handler: Arc<T>,
}

/// A [`GothamService`] connected to an IPv4 or IPv6 client
///
/// [`GothamService`]: ./struct.GothamService.html
pub struct InetGothamService<T> {
    handler: Arc<T>,
    client_addr: SocketAddr,
}

impl<T> GothamService<T>
where
    T: NewHandler + 'static,
{
    /// Convert a handler factory to a service.
    pub fn new(handler: T) -> GothamService<T> {
        GothamService {
            handler: Arc::new(handler),
        }
    }

    /// Connect to an IP or IP6 client
    pub fn connect(&self, client_addr: SocketAddr) -> InetGothamService<T> {
        InetGothamService {
            handler: self.handler.clone(),
            client_addr: client_addr,
        }
    }
}

impl<T> Service<Request<Body>> for GothamService<T>
where
    T: NewHandler,
{
    type Response = Response<Body>;
    type Error = failure::Compat<failure::Error>; // :Into<Box<StdError + Send + Sync>>
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(
        &mut self,
        _cx: &mut task::Context<'_>,
    ) -> Poll<std::result::Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let state = State::new();

        dispatch_request(&*self.handler, state, req)
    }
}

impl<T> Service<Request<Body>> for InetGothamService<T>
where
    T: NewHandler + 'static,
{
    type Response = Response<Body>;
    type Error = failure::Compat<failure::Error>;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut task::Context) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let mut state = State::new();
        put_client_addr(&mut state, self.client_addr);
        dispatch_request(&*self.handler, state, req)
    }
}

/// Perform the last parts of request dispatch, which are common to all service implementations.
///
/// The incoming request is disassembled into its parts, which are inserted into the State,
/// and then the handler itself is invoked.
fn dispatch_request<'a, H>(
    handler: &H,
    mut state: State,
    req: Request<Body>,
) -> Pin<
    Box<dyn Future<Output = Result<Response<Body>, failure::Compat<failure::Error>>> + Send + 'a>,
>
where
    H: NewHandler + 'a,
{
    let (
        request::Parts {
            method,
            uri,
            version,
            headers,
            //extensions?
            ..
        },
        body,
    ) = req.into_parts();

    state.put(RequestPathSegments::new(uri.path()));
    state.put(method);
    state.put(uri);
    state.put(version);
    state.put(headers);
    state.put(body);

    {
        let request_id = set_request_id(&mut state);
        debug!(
            "[DEBUG][{}][Thread][{:?}]",
            request_id,
            thread::current().id(),
        );
    };

    trap::call_handler(handler, AssertUnwindSafe(state))
}

#[cfg(test)]
mod tests {
    use super::*;

    use hyper::{Body, StatusCode};

    use crate::helpers::http::response::create_empty_response;
    use crate::router::builder::*;
    use crate::state::State;

    fn handler(state: State) -> (State, Response<Body>) {
        let res = create_empty_response(&state, StatusCode::ACCEPTED);
        (state, res)
    }

    #[test]
    fn new_handler_closure() {
        let service = GothamService::new(|| Ok(handler));

        let req = Request::get("http://localhost/")
            .body(Body::empty())
            .unwrap();
        let f = service
            .connect("127.0.0.1:10000".parse().unwrap())
            .call(req);
        let response = futures::executor::block_on(f).unwrap();
        assert_eq!(response.status(), StatusCode::ACCEPTED);
    }

    #[test]
    fn router() {
        let router = build_simple_router(|route| {
            route.get("/").to(handler);
        });

        let service = GothamService::new(router);

        let req = Request::get("http://localhost/")
            .body(Body::empty())
            .unwrap();
        let f = service
            .connect("127.0.0.1:10000".parse().unwrap())
            .call(req);
        let response = futures::executor::block_on(f).unwrap();
        assert_eq!(response.status(), StatusCode::ACCEPTED);
    }
}
