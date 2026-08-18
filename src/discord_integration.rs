use std::time::Duration;

use futures::{sink::SinkExt, stream::StreamExt};
use serde::Deserialize;
use serde_json::json;
use thiserror::Error;
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[derive(Error, Debug)]
pub enum GetCurrentTokenError {
    #[error("WebSocket Error: {0}")]
    WebSocketError(#[from] tokio_tungstenite::tungstenite::Error),

    #[error("Spotify Account is not associated")]
    SpotifyAccountIsNotAssociated,

    #[error("Unexpected WebSocket Close")]
    UnexpectedWebSocketClose,

    #[error("Discord READY event timeout")]
    Timeout,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SpotifyCredential {
    pub id: String,
    pub access_token: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type")]
pub enum ConnectedAccount {
    Spotify(SpotifyCredential),

    #[serde(other)]
    Other,
}

#[derive(Debug, Deserialize)]
pub struct Ready {
    pub connected_accounts: Vec<ConnectedAccount>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "t", content = "d")]
#[serde(rename_all = "UPPERCASE")]
pub enum Response {
    Ready(Ready),
}

pub async fn get_current_spotify_token(
    discord_token: &str,
) -> Result<SpotifyCredential, GetCurrentTokenError> {
    tracing::info!("Get spotify credential by Discord WebSocket...");

    let (mut ws, _) = connect_async("wss://gateway.discord.gg/")
        .await
        .map_err(GetCurrentTokenError::WebSocketError)?;

    let req = json!({
        "op": 2,
        "d": {
            "token": discord_token,
            "properties": {},
            "compress": false,
        },
    });

    ws.send(Message::Text(req.to_string().into()))
        .await
        .map_err(GetCurrentTokenError::WebSocketError)?;

    let mut timeout = tokio::time::interval(Duration::from_secs(10));
    let _consume_first_tick = timeout.tick().await;

    loop {
        tokio::select! {
            message = ws.next() => {
                match message
                    .unwrap()
                    .map_err(GetCurrentTokenError::WebSocketError)?
                {
                    Message::Text(v) => {
                        if let Ok(Response::Ready(v)) = serde_json::from_str::<Response>(&v) {
                            let cred = v
                                .connected_accounts
                                .into_iter()
                                .find_map(|v| match v {
                                    ConnectedAccount::Spotify(v) => Some(v),
                                    ConnectedAccount::Other => None,
                                })
                                .ok_or(GetCurrentTokenError::SpotifyAccountIsNotAssociated)?;

                            return Ok(cred);
                        }
                    }
                    Message::Close(_) =>
                        return Err(GetCurrentTokenError::UnexpectedWebSocketClose),
                    _ => {},
                }
            }
            _ = timeout.tick() => return Err(GetCurrentTokenError::Timeout),
        }
    }
}

#[derive(Error, Debug)]
pub enum RenewError {
    #[error("Request Error: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("Unexpected StatusCode {0}")]
    UnexpectedStatusCode(reqwest::StatusCode),

    #[error("Json Parse: {0}")]
    JsonParse(#[from] serde_json::Error),
}

#[derive(Debug, Deserialize)]
pub struct SpotifyAccessTokenApiResponse {
    pub access_token: String,
}

pub async fn renew_spotify_token(
    discord_token: &str,
    spotify_id: &str,
) -> Result<String, RenewError> {
    tracing::info!("Renew spotify credential by Discord API...");
    let resp = reqwest::Client::new()
        .get(format!(
            "https://discord.com/api/v9/users/@me/connections/spotify/{spotify_id}/access-token"
        ))
        .header(reqwest::header::AUTHORIZATION, discord_token)
        .send()
        .await
        .map_err(RenewError::RequestError)?;

    match resp.status() {
        reqwest::StatusCode::OK => {}
        code => {
            return Err(RenewError::UnexpectedStatusCode(code));
        }
    }

    let resp = resp.text().await.map_err(RenewError::RequestError)?;

    let resp: SpotifyAccessTokenApiResponse =
        serde_json::from_str(&resp).map_err(RenewError::JsonParse)?;

    Ok(resp.access_token)
}

#[derive(Error, Debug)]
pub enum GetAvailableTokenError {
    #[error("Get current token error: {0}")]
    GetCurrentTokenError(#[from] GetCurrentTokenError),

    #[error("Token availability check error: {0}")]
    TokenAvailabilityCheckError(#[from] crate::TokenAvailabilityCheckError),

    #[error("Renew Error: {0}")]
    RenewError(#[from] RenewError),
}

pub async fn get_available_spotify_token(
    discord_token: &str,
) -> Result<SpotifyCredential, GetAvailableTokenError> {
    let mut cred = get_current_spotify_token(discord_token)
        .await
        .map_err(GetAvailableTokenError::GetCurrentTokenError)?;

    if crate::is_token_available(&cred.access_token)
        .await
        .map_err(GetAvailableTokenError::TokenAvailabilityCheckError)?
    {
        return Ok(cred);
    }

    cred.access_token = renew_spotify_token(discord_token, &cred.id)
        .await
        .map_err(GetAvailableTokenError::RenewError)?;

    Ok(cred)
}
