#![doc = include_str!("../Readme.md")]
#![warn(clippy::all, clippy::pedantic, clippy::cargo, clippy::nursery)]

use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpListener};
use std::path::PathBuf;
use std::time::Duration;

use axum::http::{Request, StatusCode};
use axum::middleware::Next;
use axum::response::Response;
use axum::routing::post;
use axum::{middleware, Json, Router};
use clap::Parser;
use cli_batteries::{await_shutdown, version};
use http::header::HeaderMap;
use tracing::{info, info_span, instrument};

#[derive(Debug, Clone, Parser)]
struct Options {}

pub async fn extract_trace_layer<B>(
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    let (parts, body) = request.into_parts();

    let span = info_span!("My app name");
    let _span = span.enter();

    cli_batteries::trace_from_headers(&parts.headers);

    let request = Request::from_parts(parts, body);

    let mut response = next.run(request).await;

    cli_batteries::trace_to_headers(response.headers_mut());

    Ok(response)
}

#[instrument]
async fn echo(Json(body): Json<serde_json::Value>) -> Json<serde_json::Value> {
    let mut headers = HeaderMap::new();
    cli_batteries::trace_to_headers(&mut headers);

    info!(?headers, "headers");

    info!("Echo {body:#}");

    Json(body)
}

#[instrument(name = "Example app")]
async fn app(_options: Options) -> eyre::Result<()> {
    let router = Router::new()
        .route("/echo", post(echo))
        .layer(middleware::from_fn(extract_trace_layer))
        .with_state(app.clone());

    let port = 3000;
    let addr = SocketAddr::new(Ipv4Addr::LOCALHOST.into(), port);

    info!("Will listen on {}", addr);
    let listener = TcpListener::bind(addr)?;

    let server = axum::Server::from_tcp(listener)?
        .serve(router.into_make_service())
        .with_graceful_shutdown(await_shutdown());

    server.await?;

    Ok(())
}

fn main() {
    cli_batteries::run(version!(mio), app);
}
