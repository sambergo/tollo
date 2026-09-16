use axum::{
    extract::{Query, State},
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use reqwest::Client;
use serde::Deserialize;
use std::sync::Arc;
use tauri;
use tokio::net::TcpListener;

struct ProxyAppState {
    client: Client,
}

#[derive(Deserialize)]
struct StreamQuery {
    url: String,
}

pub async fn start_proxy_server() -> Result<u16, Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .user_agent("Mozilla/5.0")
        .build()?;

    let state = Arc::new(ProxyAppState { client });

    let router = Router::new()
        .route("/stream", get(stream_handler))
        .with_state(state);

    tokio::spawn(async move {
        axum::serve(listener, router)
            .await
            .expect("stream proxy server stopped unexpectedly");
    });

    Ok(port)
}

async fn stream_handler(
    State(state): State<Arc<ProxyAppState>>,
    Query(query): Query<StreamQuery>,
) -> Response {
    let upstream = match state.client.get(&query.url).send().await {
        Ok(r) => r,
        Err(e) => {
            return (StatusCode::BAD_GATEWAY, e.to_string()).into_response();
        }
    };

    let content_type = upstream
        .headers()
        .get(header::CONTENT_TYPE)
        .cloned()
        .unwrap_or_else(|| HeaderValue::from_static("video/mp2t"));

    let status = upstream.status();
    let byte_stream = upstream.bytes_stream();
    let body = axum::body::Body::from_stream(byte_stream);

    axum::response::Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .header(header::CACHE_CONTROL, "no-cache")
        .body(body)
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

#[tauri::command]
pub fn get_proxy_port(state: tauri::State<crate::state::ProxyState>) -> u16 {
    state.port
}
