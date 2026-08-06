//! gRPC propagation of the SIPLEI operation, mirroring `operation.rs`'s AMQP
//! mechanism on a second transport with the same header name
//! ([`OPERATION_HEADER`]). Lives here rather than in a dedicated gRPC crate
//! because that constant and the task-local it reads from are already
//! defined here — `operation.rs`'s own doc comment declared gRPC a first-class
//! consumer before this module existed.
//!
//! Two `tower::Layer`s, one per side of a call:
//!
//! - [`AttachOperationLayer`] — client side. Wraps a tonic `Channel` so every
//!   outgoing call carries the operation bound to the current task, with no
//!   per-RPC boilerplate.
//! - [`PropagateOperationLayer`] — server side. Wraps a tonic service so every
//!   incoming call opens the task-local scope for the duration of the
//!   handler, before the handler runs. A handler just calls
//!   [`current_operation`](crate::operation::current_operation) — it never
//!   touches metadata.
//!
//! Both operate on the raw `http::Request`/`http::Response` level tonic's
//! generated clients and services are built on, so they wire in once at
//! construction time and apply to every method automatically.

use crate::operation::{current_operation, with_operation, OPERATION_HEADER};
use http::{HeaderValue, Request, Response};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tower::{Layer, Service};

/// Client-side layer: attaches `x-operation-id` to every outgoing request
/// from whatever operation is bound to the current task. Silent no-op when
/// none is bound — same permissive-migration behavior as the AMQP side.
#[derive(Debug, Clone, Default)]
pub struct AttachOperationLayer;

impl<S> Layer<S> for AttachOperationLayer {
    type Service = AttachOperationService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AttachOperationService { inner }
    }
}

#[derive(Debug, Clone)]
pub struct AttachOperationService<S> {
    inner: S,
}

impl<S, ReqBody> Service<Request<ReqBody>> for AttachOperationService<S>
where
    S: Service<Request<ReqBody>>,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = S::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request<ReqBody>) -> Self::Future {
        if let Some(operation_id) = current_operation() {
            if let Ok(value) = HeaderValue::from_str(&operation_id) {
                req.headers_mut().insert(OPERATION_HEADER, value);
            }
            // An operation_id containing bytes invalid for a header value
            // would be a bug upstream (it should be an opaque id), not
            // something to fail the call over. Same silent-skip stance as a
            // missing operation.
        }
        self.inner.call(req)
    }
}

/// Server-side layer: reads `x-operation-id` from the incoming request and
/// runs the wrapped service's `call` inside that scope, so
/// [`current_operation`](crate::operation::current_operation) resolves
/// correctly anywhere inside the handler — including if the handler itself
/// makes further gRPC calls that need [`AttachOperationLayer`] to keep the
/// chain going.
#[derive(Debug, Clone, Default)]
pub struct PropagateOperationLayer;

impl<S> Layer<S> for PropagateOperationLayer {
    type Service = PropagateOperationService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        PropagateOperationService { inner }
    }
}

#[derive(Debug, Clone)]
pub struct PropagateOperationService<S> {
    inner: S,
}

impl<S, ReqBody, ResBody> Service<Request<ReqBody>> for PropagateOperationService<S>
where
    S: Service<Request<ReqBody>, Response = Response<ResBody>> + Clone + Send + 'static,
    S::Future: Send + 'static,
    ReqBody: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        let operation_id = req
            .headers()
            .get(OPERATION_HEADER)
            .and_then(|v| v.to_str().ok())
            .map(str::to_string);

        // Clone-and-swap: the classic tower pattern for a service that needs
        // to move `self` into an async block while `&mut self` is still the
        // signature tower requires. The clone must be ready to poll — tonic
        // services are, being either stateless or Arc-backed.
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        Box::pin(async move { with_operation(operation_id, inner.call(req)).await })
    }
}

#[cfg(test)]
mod test_grpc {
    use super::*;
    use crate::operation::with_operation;
    use http_body_util::Empty;
    use std::convert::Infallible;
    use tower::service_fn;
    use tower::ServiceExt;

    fn empty_request() -> Request<Empty<bytes::Bytes>> {
        Request::new(Empty::new())
    }

    #[tokio::test]
    async fn attach_sets_the_header_from_the_current_task() {
        let recorded = std::sync::Arc::new(std::sync::Mutex::new(None));
        let recorded_clone = recorded.clone();

        let inner = service_fn(move |req: Request<Empty<bytes::Bytes>>| {
            let recorded = recorded_clone.clone();
            async move {
                *recorded.lock().unwrap() = req
                    .headers()
                    .get(OPERATION_HEADER)
                    .and_then(|v| v.to_str().ok())
                    .map(str::to_string);
                Ok::<_, Infallible>(Response::new(Empty::<bytes::Bytes>::new()))
            }
        });

        let mut svc = AttachOperationLayer.layer(inner);

        with_operation(Some("op-123".to_string()), async {
            svc.ready()
                .await
                .unwrap()
                .call(empty_request())
                .await
                .unwrap();
        })
        .await;

        assert_eq!(*recorded.lock().unwrap(), Some("op-123".to_string()));
    }

    #[tokio::test]
    async fn attach_leaves_the_header_absent_without_a_bound_operation() {
        let recorded = std::sync::Arc::new(std::sync::Mutex::new(Some("untouched".to_string())));
        let recorded_clone = recorded.clone();

        let inner = service_fn(move |req: Request<Empty<bytes::Bytes>>| {
            let recorded = recorded_clone.clone();
            async move {
                *recorded.lock().unwrap() = req
                    .headers()
                    .get(OPERATION_HEADER)
                    .and_then(|v| v.to_str().ok())
                    .map(str::to_string);
                Ok::<_, Infallible>(Response::new(Empty::<bytes::Bytes>::new()))
            }
        });

        let mut svc = AttachOperationLayer.layer(inner);
        svc.ready()
            .await
            .unwrap()
            .call(empty_request())
            .await
            .unwrap();

        assert_eq!(*recorded.lock().unwrap(), None);
    }

    #[tokio::test]
    async fn propagate_opens_the_scope_the_handler_reads() {
        let inner = service_fn(|_req: Request<Empty<bytes::Bytes>>| async move {
            let seen = current_operation();
            Ok::<_, Infallible>(Response::new(Empty::<bytes::Bytes>::new()).map(|_| seen))
        });

        let mut svc = PropagateOperationLayer.layer(inner);

        let mut req = empty_request();
        req.headers_mut().insert(
            OPERATION_HEADER,
            HeaderValue::from_static("op-from-metadata"),
        );

        let resp = svc.ready().await.unwrap().call(req).await.unwrap();
        assert_eq!(resp.into_body(), Some("op-from-metadata".to_string()));
    }

    #[tokio::test]
    async fn propagate_binds_none_without_the_header() {
        let inner = service_fn(|_req: Request<Empty<bytes::Bytes>>| async move {
            let seen = current_operation();
            Ok::<_, Infallible>(Response::new(Empty::<bytes::Bytes>::new()).map(|_| seen))
        });

        let mut svc = PropagateOperationLayer.layer(inner);
        let resp = svc
            .ready()
            .await
            .unwrap()
            .call(empty_request())
            .await
            .unwrap();

        assert_eq!(resp.into_body(), None);
    }

    // The chain that matters end to end: a server receives an operation,
    // and a further outgoing call made from inside that handler carries it
    // onward without the handler doing anything special.
    #[tokio::test]
    async fn propagate_then_attach_forwards_the_operation_across_a_hop() {
        let downstream_recorded = std::sync::Arc::new(std::sync::Mutex::new(None));
        let downstream_recorded_clone = downstream_recorded.clone();

        let downstream = service_fn(move |req: Request<Empty<bytes::Bytes>>| {
            let recorded = downstream_recorded_clone.clone();
            async move {
                *recorded.lock().unwrap() = req
                    .headers()
                    .get(OPERATION_HEADER)
                    .and_then(|v| v.to_str().ok())
                    .map(str::to_string);
                Ok::<_, Infallible>(Response::new(Empty::<bytes::Bytes>::new()))
            }
        });
        let downstream_client = AttachOperationLayer.layer(downstream);

        let handler = tower::service_fn(move |_req: Request<Empty<bytes::Bytes>>| {
            let mut downstream_client = downstream_client.clone();
            async move {
                downstream_client
                    .ready()
                    .await
                    .unwrap()
                    .call(Request::new(Empty::new()))
                    .await
                    .unwrap();
                Ok::<_, Infallible>(Response::new(Empty::<bytes::Bytes>::new()))
            }
        });
        let mut incoming_server = PropagateOperationLayer.layer(handler);

        let mut req = empty_request();
        req.headers_mut().insert(
            OPERATION_HEADER,
            HeaderValue::from_static("op-hopping-through"),
        );
        incoming_server
            .ready()
            .await
            .unwrap()
            .call(req)
            .await
            .unwrap();

        assert_eq!(
            *downstream_recorded.lock().unwrap(),
            Some("op-hopping-through".to_string())
        );
    }
}
