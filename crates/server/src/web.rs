//! The built web client (`web/dist`), served from the same origin as the API
//! so cookies, CSRF and the socket's Origin check all see one site.

use std::path::Path;

use axum::Router;
use axum::extract::Request;
use axum::http::{HeaderValue, header};
use axum::middleware::{self, Next};
use axum::response::Response;
use tower_http::services::ServeDir;

/// What pages may load: only our own scripts, styles and connections (the
/// game socket included). Svelte sets some inline style attributes.
pub const CONTENT_SECURITY_POLICY: &str = "default-src 'self'; script-src 'self'; \
     style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self'; \
     object-src 'none'; base-uri 'none'; form-action 'self'; frame-ancestors 'none'";

/// Serves `dir`'s files for every path `router` doesn't answer; `/` is its
/// `index.html`. Unknown files are 404s (the client has no routes of its own).
pub fn with_web(router: Router, dir: &Path) -> Router {
    let files = Router::new()
        .fallback_service(ServeDir::new(dir))
        .layer(middleware::from_fn(headers));
    router.fallback_service(files)
}

/// Vite names built assets by content hash, so they never change and can be
/// cached for good; everything else (the index) is checked on every load.
async fn headers(request: Request, next: Next) -> Response {
    let immutable = request.uri().path().starts_with("/assets/");
    let mut response = next.run(request).await;
    let cache = if immutable && response.status().is_success() {
        "public, max-age=31536000, immutable"
    } else {
        "no-cache"
    };
    let headers = response.headers_mut();
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static(cache));
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(CONTENT_SECURITY_POLICY),
    );
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("same-origin"),
    );
    response
}
