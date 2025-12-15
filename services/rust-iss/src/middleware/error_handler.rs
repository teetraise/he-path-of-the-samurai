use axum::{
    body::Body,
    extract::Request,
    middleware::Next,
    response::Response,
};
use tracing::info;

/// Middleware for logging all requests and responses
pub async fn error_logging_middleware(req: Request, next: Next) -> Response {
    let method = req.method().clone();
    let uri = req.uri().clone();

    info!("Incoming request: {} {}", method, uri);

    let response = next.run(req).await;

    info!("Response status: {}", response.status());

    response
}
