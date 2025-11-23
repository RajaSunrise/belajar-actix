use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse, body::EitherBody,
};
use futures::future::{ok, Ready};
use futures::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll};
use crate::services::redis::RedisPool;
use redis::AsyncCommands;

#[derive(Clone)]
pub struct RateLimit {
    pub pool: RedisPool,
}

impl<S, B> Transform<S, ServiceRequest> for RateLimit
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Transform = RateLimitMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(RateLimitMiddleware {
            service: Rc::new(service),
            pool: self.pool.clone(),
        })
    }
}

pub struct RateLimitMiddleware<S> {
    service: Rc<S>,
    pool: RedisPool,
}

impl<S, B> Service<ServiceRequest> for RateLimitMiddleware<S>
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
        let pool = self.pool.clone();

        let ip = req.peer_addr().map(|a| a.ip().to_string()).unwrap_or_else(|| "unknown".to_string());

        Box::pin(async move {
            let mut con = match pool.get_multiplexed_async_connection().await {
                Ok(c) => c,
                Err(e) => {
                    log::error!("Redis error: {}", e);
                    // Fail open
                    let res = srv.call(req).await?;
                    return Ok(res.map_into_left_body());
                }
            };

            let key = format!("ratelimit:{}", ip);
            let limit = 100; // 100 req
            let ttl = 60; // 60 sec

            let script = redis::Script::new(r#"
                let current = redis.call("INCR", KEYS[1])
                if tonumber(current) == 1 then
                    redis.call("EXPIRE", KEYS[1], ARGV[1])
                end
                return current
            "#);

            let count: u64 = match script.key(&key).arg(ttl).invoke_async(&mut con).await {
                Ok(c) => c,
                Err(e) => {
                    log::error!("Redis script error: {}", e);
                    let res = srv.call(req).await?;
                    return Ok(res.map_into_left_body());
                }
            };

            if count > limit {
                 let res = HttpResponse::TooManyRequests().body("Rate Limit Exceeded").map_into_right_body();
                 return Ok(req.into_response(res));
            }

            let res = srv.call(req).await?;
            Ok(res.map_into_left_body())
        })
    }
}
