use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures_util::future::{ok, LocalBoxFuture, Ready};
use std::{rc::Rc, time::Instant};

pub struct RequestLoggingMiddleware;

impl<S, B> Transform<S, ServiceRequest> for RequestLoggingMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RequestLoggingMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(RequestLoggingMiddlewareService {
            service: Rc::new(service),
        })
    }
}

pub struct RequestLoggingMiddlewareService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for RequestLoggingMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);
        let started_at = Instant::now();

        let request_id = req
            .headers()
            .get("x-request-id")
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned)
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        let method = req.method().to_string();
        let path = req.path().to_owned();
        let query = req.query_string().to_owned();
        let peer_addr = req
            .peer_addr()
            .map(|addr| addr.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        let user_agent = req
            .headers()
            .get("user-agent")
            .and_then(|value| value.to_str().ok())
            .unwrap_or("unknown")
            .to_owned();

        let route = if query.is_empty() {
            path.clone()
        } else {
            format!("{path}?{query}")
        };

        log::info!(
            "request started id={} method={} route={} peer={} user_agent={:?}",
            request_id,
            method,
            route,
            peer_addr,
            user_agent
        );

        Box::pin(async move {
            match service.call(req).await {
                Ok(response) => {
                    let status = response.status();
                    let duration_ms = started_at.elapsed().as_millis();

                    if status.is_server_error() {
                        log::error!(
                            "request finished id={} method={} route={} status={} duration_ms={}",
                            request_id,
                            method,
                            route,
                            status.as_u16(),
                            duration_ms
                        );
                    } else if status.is_client_error() {
                        log::warn!(
                            "request finished id={} method={} route={} status={} duration_ms={}",
                            request_id,
                            method,
                            route,
                            status.as_u16(),
                            duration_ms
                        );
                    } else {
                        log::info!(
                            "request finished id={} method={} route={} status={} duration_ms={}",
                            request_id,
                            method,
                            route,
                            status.as_u16(),
                            duration_ms
                        );
                    }

                    Ok(response)
                }
                Err(error) => {
                    log::error!(
                        "request failed id={} method={} route={} duration_ms={} error={}",
                        request_id,
                        method,
                        route,
                        started_at.elapsed().as_millis(),
                        error
                    );

                    Err(error)
                }
            }
        })
    }
}
