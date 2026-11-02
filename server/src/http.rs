//! HTTP and WebSocket edge. Handlers do not touch the valley; they send
//! messages to the host task and wait for an acknowledgement.

use crate::host::{Ack, Clock, Host, Joined};
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::json;
use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::{broadcast, mpsc, oneshot};
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

pub enum Msg {
    Join {
        name: String,
        reply: oneshot::Sender<Result<Joined, String>>,
    },
    Submit {
        token: String,
        command_id: String,
        line: String,
        window: Option<u64>,
        reply: oneshot::Sender<Ack>,
    },
    Observe {
        token: String,
        reply: oneshot::Sender<Result<serde_json::Value, String>>,
    },
    Clock {
        reply: oneshot::Sender<Clock>,
    },
    Advance {
        steps: u32,
        reply: oneshot::Sender<Result<u64, String>>,
    },
    Shutdown,
}

#[derive(Clone)]
pub struct App {
    pub tx: mpsc::Sender<Msg>,
    pub ticks: broadcast::Sender<u64>,
}

pub fn router(app: App, static_dir: PathBuf) -> Router {
    let api = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/v1/join", post(join))
        .route("/v1/intent", post(intent))
        .route("/v1/observe", get(observe))
        .route("/v1/clock", get(clock))
        .route("/v1/advance", post(advance))
        .route("/v1/ws", get(ws))
        .with_state(app);
    if static_dir.is_dir() {
        api.fallback_service(ServeDir::new(static_dir))
            .layer(CorsLayer::permissive())
    } else {
        api.fallback(get(|| async {
            (
                StatusCode::NOT_FOUND,
                "The web client has not been built. From web/, run npm install && npm run build.",
            )
        }))
        .layer(CorsLayer::permissive())
    }
}

pub async fn run_host(mut host: Host, mut rx: mpsc::Receiver<Msg>, ticks: broadcast::Sender<u64>) {
    let manual = host.manual;
    loop {
        if manual {
            match rx.recv().await {
                Some(msg) => {
                    if handle(&mut host, msg, &ticks).await {
                        break;
                    }
                }
                None => break,
            }
        } else {
            tokio::select! {
                biased;
                msg = rx.recv() => {
                    match msg {
                        Some(msg) => {
                            if handle(&mut host, msg, &ticks).await {
                                break;
                            }
                        }
                        None => break,
                    }
                }
                _ = tokio::time::sleep(window_remaining(&host)) => {
                    if let Err(err) = host.seal() {
                        eprintln!("seal failed: {err}");
                        break;
                    }
                    let _ = ticks.send(host.world().sim_time());
                }
            }
        }
    }
    if let Err(err) = host.snapshot() {
        eprintln!("final snapshot failed: {err}");
    }
}

fn window_remaining(host: &Host) -> Duration {
    let millis = host.clock().closes_in_ms.unwrap_or(0);
    Duration::from_millis(millis.max(1))
}

async fn handle(host: &mut Host, msg: Msg, ticks: &broadcast::Sender<u64>) -> bool {
    match msg {
        Msg::Join { name, reply } => {
            let _ = reply.send(host.join(&name));
        }
        Msg::Submit {
            token,
            command_id,
            line,
            window,
            reply,
        } => {
            let _ = reply.send(host.submit(&token, &command_id, &line, window));
        }
        Msg::Observe { token, reply } => {
            let value = host
                .observe(&token)
                .map(|obs| serde_json::to_value(obs).unwrap_or(json!({})))
                .ok_or_else(|| "Unknown token.".to_string());
            let _ = reply.send(value);
        }
        Msg::Clock { reply } => {
            let _ = reply.send(host.clock());
        }
        Msg::Advance { steps, reply } => {
            let result = host.advance(steps);
            if result.is_ok() {
                let _ = ticks.send(host.world().sim_time());
            }
            let _ = reply.send(result);
        }
        Msg::Shutdown => return true,
    }
    false
}

#[derive(Deserialize)]
struct JoinBody {
    name: String,
}

async fn join(State(app): State<App>, Json(body): Json<JoinBody>) -> impl IntoResponse {
    let (reply, rx) = oneshot::channel();
    if app
        .tx
        .send(Msg::Join {
            name: body.name,
            reply,
        })
        .await
        .is_err()
    {
        return err(StatusCode::SERVICE_UNAVAILABLE, "The valley is shut.");
    }
    match rx.await {
        Ok(Ok(joined)) => (StatusCode::OK, Json(json!(joined))).into_response(),
        Ok(Err(message)) => err(StatusCode::BAD_REQUEST, &message),
        Err(_) => err(
            StatusCode::SERVICE_UNAVAILABLE,
            "The valley dropped the request.",
        ),
    }
}

#[derive(Deserialize)]
struct IntentBody {
    command_id: String,
    line: String,
    window: Option<u64>,
}

async fn intent(
    State(app): State<App>,
    headers: HeaderMap,
    Json(body): Json<IntentBody>,
) -> impl IntoResponse {
    let Some(token) = bearer(&headers) else {
        return err(
            StatusCode::UNAUTHORIZED,
            "Send Authorization: Bearer <token>.",
        );
    };
    let (reply, rx) = oneshot::channel();
    if app
        .tx
        .send(Msg::Submit {
            token,
            command_id: body.command_id,
            line: body.line,
            window: body.window,
            reply,
        })
        .await
        .is_err()
    {
        return err(StatusCode::SERVICE_UNAVAILABLE, "The valley is shut.");
    }
    match rx.await {
        Ok(ack) => {
            let status = if ack.ok {
                StatusCode::OK
            } else {
                StatusCode::BAD_REQUEST
            };
            (status, Json(ack)).into_response()
        }
        Err(_) => err(
            StatusCode::SERVICE_UNAVAILABLE,
            "The valley dropped the request.",
        ),
    }
}

async fn observe(State(app): State<App>, headers: HeaderMap) -> impl IntoResponse {
    let Some(token) = bearer(&headers) else {
        return err(
            StatusCode::UNAUTHORIZED,
            "Send Authorization: Bearer <token>.",
        );
    };
    let (reply, rx) = oneshot::channel();
    if app.tx.send(Msg::Observe { token, reply }).await.is_err() {
        return err(StatusCode::SERVICE_UNAVAILABLE, "The valley is shut.");
    }
    match rx.await {
        Ok(Ok(value)) => (StatusCode::OK, Json(value)).into_response(),
        Ok(Err(message)) => err(StatusCode::UNAUTHORIZED, &message),
        Err(_) => err(
            StatusCode::SERVICE_UNAVAILABLE,
            "The valley dropped the request.",
        ),
    }
}

async fn clock(State(app): State<App>) -> impl IntoResponse {
    let (reply, rx) = oneshot::channel();
    if app.tx.send(Msg::Clock { reply }).await.is_err() {
        return err(StatusCode::SERVICE_UNAVAILABLE, "The valley is shut.");
    }
    match rx.await {
        Ok(clock) => (StatusCode::OK, Json(clock)).into_response(),
        Err(_) => err(
            StatusCode::SERVICE_UNAVAILABLE,
            "The valley dropped the request.",
        ),
    }
}

#[derive(Deserialize)]
struct AdvanceBody {
    #[serde(default = "one")]
    steps: u32,
}

fn one() -> u32 {
    1
}

async fn advance(State(app): State<App>, Json(body): Json<AdvanceBody>) -> impl IntoResponse {
    let (reply, rx) = oneshot::channel();
    if app
        .tx
        .send(Msg::Advance {
            steps: body.steps,
            reply,
        })
        .await
        .is_err()
    {
        return err(StatusCode::SERVICE_UNAVAILABLE, "The valley is shut.");
    }
    match rx.await {
        Ok(Ok(sim_time)) => (StatusCode::OK, Json(json!({ "sim_time": sim_time }))).into_response(),
        Ok(Err(message)) => err(StatusCode::BAD_REQUEST, &message),
        Err(_) => err(
            StatusCode::SERVICE_UNAVAILABLE,
            "The valley dropped the request.",
        ),
    }
}

#[derive(Deserialize)]
struct WsQuery {
    token: String,
}

async fn ws(
    State(app): State<App>,
    Query(query): Query<WsQuery>,
    upgrade: WebSocketUpgrade,
) -> impl IntoResponse {
    upgrade.on_upgrade(move |socket| socket_loop(socket, app, query.token))
}

async fn socket_loop(mut socket: WebSocket, app: App, token: String) {
    let mut ticks = app.ticks.subscribe();
    if let Some(payload) = observe_json(&app, &token).await {
        let _ = socket.send(Message::Text(payload)).await;
    }
    loop {
        tokio::select! {
            incoming = socket.recv() => {
                match incoming {
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Ok(Message::Text(text))) => {
                        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) {
                            if value.get("line").is_some() {
                                let command_id = value.get("command_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                let line = value.get("line").and_then(|v| v.as_str()).unwrap_or("").to_string();
                                let window = value.get("window").and_then(|v| v.as_u64());
                                let (reply, rx) = oneshot::channel();
                                if app.tx.send(Msg::Submit { token: token.clone(), command_id, line, window, reply }).await.is_err() {
                                    break;
                                }
                                if let Ok(ack) = rx.await {
                                    let _ = socket.send(Message::Text(serde_json::to_string(&ack).unwrap_or_default())).await;
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ = ticks.recv() => {
                if let Some(payload) = observe_json(&app, &token).await {
                    if socket.send(Message::Text(payload)).await.is_err() {
                        break;
                    }
                }
            }
        }
    }
}

async fn observe_json(app: &App, token: &str) -> Option<String> {
    let (reply, rx) = oneshot::channel();
    app.tx
        .send(Msg::Observe {
            token: token.to_string(),
            reply,
        })
        .await
        .ok()?;
    let value = rx.await.ok()?.ok()?;
    serde_json::to_string(&json!({ "type": "observe", "observation": value })).ok()
}

fn bearer(headers: &HeaderMap) -> Option<String> {
    let value = headers
        .get(axum::http::header::AUTHORIZATION)?
        .to_str()
        .ok()?;
    value
        .strip_prefix("Bearer ")
        .map(|token| token.trim().to_string())
}

fn err(status: StatusCode, message: &str) -> axum::response::Response {
    (status, Json(json!({ "ok": false, "message": message }))).into_response()
}
