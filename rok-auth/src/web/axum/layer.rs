//! Tower middleware that injects Auth into request extensions.

use std::sync::Arc;
use std::task::{Context, Poll};

use axum::extract::Request;
use tower::{Layer, Service};

use crate::Auth;

#[derive(Clone)]
pub struct AuthLayer {
    auth: Arc<Auth>,
}

impl AuthLayer {
    pub fn new(auth: Auth) -> Self {
        Self {
            auth: Arc::new(auth),
        }
    }
}

impl<S> Layer<S> for AuthLayer {
    type Service = AuthMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AuthMiddleware {
            inner,
            auth: Arc::clone(&self.auth),
        }
    }
}

#[derive(Clone)]
pub struct AuthMiddleware<S> {
    inner: S,
    auth: Arc<Auth>,
}

impl<S, B> Service<Request<B>> for AuthMiddleware<S>
where
    S: Service<Request<B>>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<B>) -> Self::Future {
        req.extensions_mut().insert(Arc::clone(&self.auth));
        self.inner.call(req)
    }
}
