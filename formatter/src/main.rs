use axum::{
    extract::{MatchedPath, Request}, response::Html, routing::{get, post}, Router
};
use formatcore::{FormatError, Formatter};
use tower_http::trace::TraceLayer;
use tracing::{debug, error, info, instrument};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
mod format;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    let formatter = format::enabled_formatter::FORMATTER;
    let app = Router::new()
        .route("/format", post(handler))
        .layer(
            TraceLayer::new_for_http()
                // Create our own span for the request and include the matched path. The matched
                // path is useful for figuring out which handler the request was routed to.
                .make_span_with(|req: &Request| {
                    let method = req.method();
                    let uri = req.uri();

                    // axum automatically adds this extension.
                    let matched_path = req
                        .extensions()
                        .get::<MatchedPath>()
                        .map(|matched_path| matched_path.as_str());

                    tracing::debug_span!("request", %method, %uri, matched_path)
                })
                // By default `TraceLayer` will log 5xx responses but we're doing our specific
                // logging of errors so disable that
                .on_failure(()),
        );
    let bind_addr = std::env::var("BIND_ADDR").expect("Error: No bind_addr was given.");
    let listener = tokio::net::TcpListener::bind(&bind_addr).await.unwrap();
    info!(
        "{formatter} formatter listening on {}",
        listener.local_addr().unwrap()
    );
    axum::serve(listener, app).await.unwrap();
}

#[instrument]
async fn handler(body: String) -> Result<String, FormatError> {
    let formatted = format::enabled_formatter::Formatter::format(&body);
    match formatted {
        Ok(body) => {
            debug!("Message formatted correctly");
            Ok(body)
        },
        Err(e) => {
            error!("Error: {e}");
            Err(e)
        }
    }
}
