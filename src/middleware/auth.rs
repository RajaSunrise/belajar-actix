use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, HttpResponse, body::EitherBody,
};
use futures::future::{ok, Ready};
use futures::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};
use crate::auth::validate_jwt;

pub struct JwtAuth;

impl<S, B> Transform<S, ServiceRequest> for JwtAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>; // Change return type to wrap B
    type Error = Error;
    type Transform = JwtAuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(JwtAuthMiddleware {
            service: Rc::new(service),
        })
    }
}

pub struct JwtAuthMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for JwtAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let srv = self.service.clone();

        let path = req.path().to_string(); // clone path
        if path.starts_with("/api/login")
            || path.starts_with("/api/register")
            || path.starts_with("/static")
            || path.starts_with("/api/refresh")
            || path == "/api/anime"
            || path == "/api/donghua"
            || path == "/api/movies"
            || path == "/api/schedule"
            || path == "/api/search"
            || path == "/api/all"
            || path == "/api/genres"
            || path == "/"
        {
             return Box::pin(async move {
                 let res = srv.call(req).await?;
                 Ok(res.map_into_left_body())
             });
        }

        let auth_header = req.headers().get("Authorization").cloned(); // clone header value

        if let Some(header) = auth_header {
            if let Ok(auth_str) = header.to_str() {
                if auth_str.starts_with("Bearer ") {
                    let token = &auth_str[7..];
                    match validate_jwt(token) {
                        Ok(claims) => {
                            req.extensions_mut().insert(claims);
                             return Box::pin(async move {
                                 let res = srv.call(req).await?;
                                 Ok(res.map_into_left_body())
                             });
                        }
                        Err(_) => {}
                    }
                }
            }
        }

        Box::pin(async move {
            let res = HttpResponse::Unauthorized().finish().map_into_right_body();
            Ok(req.into_response(res))
        })
    }
}
