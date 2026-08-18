use std::sync::Arc;
use std::time::Duration;

use futures::{sink::SinkExt, stream::StreamExt};
use thiserror::Error;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

pub mod api;
#[cfg(feature = "discord_integration")]
pub mod discord_integration;

pub use async_trait;
pub use serde_json;

#[derive(Error, Debug)]
pub enum TokenAvailabilityCheckError {
    #[error("Request Error: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("Unexpected StatusCode {0}")]
    UnexpectedStatusCode(reqwest::StatusCode),
}

pub async fn is_token_available(token: &str) -> Result<bool, TokenAvailabilityCheckError> {
    let resp = reqwest::Client::new()
        .get("https://api.spotify.com/v1/me/player/devices")
        .bearer_auth(token)
        .send()
        .await
        .map_err(TokenAvailabilityCheckError::RequestError)?;

    match resp.status() {
        code if code == reqwest::StatusCode::OK => Ok(true),
        code if code == reqwest::StatusCode::UNAUTHORIZED => Ok(false),
        code => Err(TokenAvailabilityCheckError::UnexpectedStatusCode(code)),
    }
}

#[derive(Error, Debug)]
pub enum PutReplyError {
    #[error("Request Error: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("Unexpected StatusCode {0}")]
    UnexpectedStatusCode(reqwest::StatusCode),

    #[error("ConnectionId is not found")]
    ConnectionIdIsNotFound,
}

async fn reply_put_request_if_needed(
    token: &str,
    msg: &api::Message,
) -> Result<bool, PutReplyError> {
    if !msg.method.as_ref().is_some_and(|m| m == "PUT") {
        return Ok(false);
    }

    if !msg.uri.starts_with("hm://pusher/v1/connections/") {
        tracing::warn!("Unknown PUT URI: {}", msg.uri);
        return Ok(false);
    }

    let Some(connection_id) = msg.headers.get("Spotify-Connection-Id") else {
        return Err(PutReplyError::ConnectionIdIsNotFound);
    };

    tracing::info!("Spotify WebSocket Connection activating...");

    let resp = reqwest::Client::new()
        .put("https://api.spotify.com/v1/me/notifications/player")
        .query(&[("connection_id", connection_id)])
        .header(reqwest::header::CONTENT_LENGTH, "0")
        .bearer_auth(token)
        .send()
        .await
        .map_err(PutReplyError::RequestError)?;

    match resp.status() {
        code if code == reqwest::StatusCode::OK => Ok(true),
        code => Err(PutReplyError::UnexpectedStatusCode(code)),
    }
}

async fn handle_wss_event(
    client: &Client,
    msg: &api::Message,
    last_active_device_id: &mut Option<String>,
) {
    if msg.uri != "wss://event" {
        return;
    }

    let Some(content_type) = msg
        .headers
        .iter()
        .find(|(key, _)| key.to_ascii_lowercase() == "content-type")
        .map(|(_, v)| v)
    else {
        tracing::warn!("wss://event doesn't provide content-type");
        return;
    };

    if content_type != "application/json" {
        tracing::warn!("Unknown wss://event content-type: {content_type}");
        return;
    }

    let Some(payloads) = &msg.payloads else {
        tracing::warn!("payloads field is not found in wss://event");
        return;
    };

    for payload in payloads {
        for handler in &client.inner.handlers {
            handler.raw_wss_event(&payload).await;
        }

        let events: api::Events = match serde_json::from_value(payload.clone()) {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!("Failed to parse events: {e}");
                return;
            }
        };

        for event in events.events {
            for handler in &client.inner.handlers {
                handler.raw_event(&event).await;
            }

            match event.r#type.as_str() {
                "DEVICE_STATE_CHANGED" => {
                    let device_state_changed: api::DeviceStateChanged =
                        match serde_json::from_value(event.event.event_body) {
                            Ok(v) => v,
                            Err(e) => {
                                tracing::warn!("Failed to parse DEVICE_STATE_CHANGED: {e}");
                                return;
                            }
                        };

                    if let Some(device_id) = last_active_device_id {
                        if device_state_changed
                            .devices
                            .iter()
                            .find(|d| d.id.as_ref().is_some_and(|id| id == &*device_id))
                            .is_none()
                        {
                            for handler in &client.inner.handlers {
                                handler.on_playback_state_update(None).await;
                            }
                        }
                    }

                    for handler in &client.inner.handlers {
                        handler
                            .device_state_changed(&device_state_changed.devices)
                            .await;
                    }
                }
                "PLAYER_STATE_CHANGED" => {
                    let player_state_changed: api::PlayerStateChanged =
                        match serde_json::from_value(event.event.event_body) {
                            Ok(v) => v,
                            Err(e) => {
                                tracing::warn!("Failed to parse PLAYER_STATE_CHANGED: {e}");
                                return;
                            }
                        };

                    if let Some(device_id) = &player_state_changed.state.device.id {
                        *last_active_device_id = Some(device_id.to_string());
                    }

                    for handler in &client.inner.handlers {
                        handler
                            .player_state_changed(&player_state_changed.state)
                            .await;

                        handler
                            .on_playback_state_update(Some(&player_state_changed.state))
                            .await;
                    }
                }
                s => tracing::warn!("Unknown message: {s}"),
            }
        }
    }
}

#[derive(Error, Debug)]
pub enum FetchPlaybackStateError {
    #[error("Request Error: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("Unexpected StatusCode {0}")]
    UnexpectedStatusCode(reqwest::StatusCode),

    #[error("Deserialize Error: {0}")]
    DeserializeError(#[from] serde_json::Error),
}

pub async fn fetch_playback_state(
    token: &str,
) -> Result<Option<api::PlaybackState>, FetchPlaybackStateError> {
    let resp = reqwest::Client::new()
        .get("https://api.spotify.com/v1/me/player")
        .bearer_auth(token)
        .send()
        .await
        .map_err(FetchPlaybackStateError::RequestError)?;

    match resp.status() {
        reqwest::StatusCode::OK => {}
        reqwest::StatusCode::NO_CONTENT => return Ok(None),
        code => return Err(FetchPlaybackStateError::UnexpectedStatusCode(code)),
    }

    let resp = resp
        .text()
        .await
        .map_err(FetchPlaybackStateError::RequestError)?;

    serde_json::from_str(&resp).map_err(FetchPlaybackStateError::DeserializeError)
}

#[async_trait::async_trait]
pub trait EventHandler: Send + Sync {
    async fn raw_message(&self, _value: &api::Message) {}
    async fn raw_wss_event(&self, _value: &serde_json::Value) {}
    async fn raw_event(&self, _value: &api::Event) {}
    async fn device_state_changed(&self, _devices: &[api::Device]) {}
    async fn player_state_changed(&self, _playback_state: &api::PlaybackState) {}
    async fn initial_playback_state(&self, _playback_state: Option<&api::PlaybackState>) {}
    async fn on_playback_state_update(&self, _playback_state: Option<&api::PlaybackState>) {}
}

pub struct ClientInner {
    token: String,
    handlers: Vec<Arc<dyn EventHandler>>,
}

#[derive(Clone)]
pub struct Client {
    inner: Arc<ClientInner>,
}

#[derive(Error, Debug)]
pub enum RunError {
    #[error("WebSocket Error: {0}")]
    WebSocketError(#[from] tokio_tungstenite::tungstenite::Error),

    #[error("Unexpected WebSocket Close")]
    UnexpectedClose,

    #[error("Unknown Message Type: {0}")]
    UnknownMessageType(#[from] serde_json::Error),

    #[error("Put Reply Error: {0}")]
    PutReplyError(#[from] PutReplyError),

    #[error("Fetch initial PlaybackState Error: {0}")]
    FetchInitialPlaybackStateError(#[from] FetchPlaybackStateError),
}

impl Client {
    pub async fn run(&self) -> Result<(), RunError> {
        tracing::info!("Connecting to Spotify WebSocket...");
        let ws_url = format!(
            "wss://dealer.spotify.com/?access_token={}",
            self.inner.token
        );

        let (ws, _) = connect_async(ws_url)
            .await
            .map_err(RunError::WebSocketError)?;

        let (mut tx, mut rx) = ws.split();
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        let _consume_first_tick = interval.tick().await;
        let mut last_active_device_id: Option<String> = None;

        loop {
            tokio::select! {
                msg = rx.next() => {
                    let msg = msg.unwrap();
                    let msg = msg.map_err(RunError::WebSocketError)?;

                    let msg = match msg {
                        Message::Text(v) => v,
                        Message::Close(_) => return Err(RunError::UnexpectedClose),
                        _ => continue,
                    };

                    let msg: api::Response = serde_json::from_str(&msg)
                        .map_err(RunError::UnknownMessageType)?;

                    let msg = match msg {
                        api::Response::Message(msg) => msg,
                        api::Response::Pong => continue,
                    };

                    for handler in &self.inner.handlers {
                        handler.raw_message(&msg).await;
                    }

                    if reply_put_request_if_needed(&self.inner.token, &msg).await.map_err(RunError::PutReplyError)? {
                        let initial_state = fetch_playback_state(&self.inner.token)
                            .await
                            .map_err(RunError::FetchInitialPlaybackStateError)?;

                        for handler in &self.inner.handlers {
                            handler.initial_playback_state(initial_state.as_ref()).await;
                            handler.on_playback_state_update(initial_state.as_ref()).await;
                        }

                        if let Some(device_id) = initial_state.map(|s| s.device.id).flatten() {
                            last_active_device_id = Some(device_id.clone());
                        }
                    }

                    handle_wss_event(&self, &msg, &mut last_active_device_id).await;
                }
                _ = interval.tick() => {
                    tx.send(
                        Message::Text(serde_json::json!({"type": "ping"}).to_string().into())
                    ).await.map_err(RunError::WebSocketError)?;
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct ClientBuilder {
    token: String,
    handlers: Vec<Arc<dyn EventHandler>>,
}

impl ClientBuilder {
    pub fn new(token: &str) -> Self {
        Self {
            token: token.to_string(),
            handlers: vec![],
        }
    }

    pub fn handler<T: EventHandler + 'static>(mut self, handler: T) -> Self {
        self.handlers.push(Arc::new(handler));
        self
    }

    pub fn build(self) -> Client {
        Client {
            inner: Arc::new(ClientInner {
                handlers: self.handlers,
                token: self.token,
            }),
        }
    }
}
