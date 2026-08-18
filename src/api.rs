use std::collections::HashMap;

use serde::Deserialize;
use serde_enum_str::Deserialize_enum_str;

#[derive(Debug, Deserialize, Clone)]
pub struct Message {
    pub headers: HashMap<String, String>,
    pub payloads: Option<Vec<serde_json::Value>>,
    pub method: Option<String>,
    pub uri: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type")]
pub enum Response {
    Message(Message),
    Pong,
}

/// Decode structures for wss://event
#[derive(Debug, Deserialize, Clone)]
pub struct User {
    pub id: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct InnerEvent {
    pub event_id: u64,

    #[serde(flatten)]
    pub event_body: serde_json::Value,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Event {
    pub event: InnerEvent,
    pub href: String,
    pub source: String,
    pub r#type: String,
    pub uri: Option<String>,
    pub user: User,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Events {
    pub events: Vec<Event>,
}

/// Device Type
/// from https://github.com/librespot-org/librespot/blob/9c7d75615fc093bdcbdb29adbce3fed38c531852/core/src/config.rs#L61-L82
/// The MIT License (MIT) Copyright (c) 2015 Paul Lietar
#[derive(Deserialize_enum_str, Debug, PartialEq, Eq, Clone, Copy)]
pub enum DeviceType {
    Unknown,
    Computer,
    Tablet,
    Smartphone,
    Speaker,
    Tv,
    Avr,
    Stb,
    AudioDongle,
    GameConsole,
    CastAudio,
    CastVideo,
    Automobile,
    Smartwatch,
    Chromebook,
    UnknownSpotify,
    CarThing,
    Obsrever,
}

/// https://developer.spotify.com/documentation/web-api/reference/get-information-about-the-users-current-playback
#[derive(Deserialize_enum_str, Debug, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum RepeatState {
    Track,
    Context,
    Off,
}

/// https://developer.spotify.com/documentation/web-api/reference/get-information-about-the-users-current-playback
#[derive(Deserialize_enum_str, Debug, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum CurrentlyPlayingType {
    Episode,
    Track,
    Ad,
    Unknown,
}

/// https://developer.spotify.com/documentation/web-api/reference/get-information-about-the-users-current-playback
#[derive(Deserialize_enum_str, Debug, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum AlbumType {
    Album,
    Single,
    Compilation,
}

/// https://developer.spotify.com/documentation/web-api/reference/get-information-about-the-users-current-playback
#[derive(Deserialize_enum_str, Debug, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum ReleaseDatePrecision {
    Year,
    Month,
    Day,
}

/// https://developer.spotify.com/documentation/web-api/reference/get-information-about-the-users-current-playback
#[derive(Deserialize_enum_str, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum RestrictionReason {
    Market,
    Product,
    Explicit,
    #[serde(other)]
    Other(String),
}

/// https://developer.spotify.com/documentation/web-api/reference/get-a-users-available-devices
#[derive(Debug, Deserialize, Clone)]
pub struct Device {
    pub id: Option<String>,
    pub is_active: bool,
    pub is_private_session: bool,
    pub is_restricted: bool,
    pub name: String,
    pub r#type: DeviceType,
    pub volume_percent: Option<u8>,
    pub supports_volume: bool,
}

/// Decode structures for DEVICE_STATE_CHANGED
/// https://developer.spotify.com/documentation/web-api/reference/get-a-users-available-devices
#[derive(Debug, Deserialize, Clone)]
pub struct DeviceStateChanged {
    pub devices: Vec<Device>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ExternalUrls {
    pub spotify: String,
}

/// https://developer.spotify.com/documentation/web-api/reference/get-information-about-the-users-current-playback
#[derive(Debug, Deserialize, Clone)]
pub struct Restrictions {
    pub reason: RestrictionReason,
}

/// https://developer.spotify.com/documentation/web-api/reference/get-information-about-the-users-current-playback
#[derive(Debug, Deserialize, Clone)]
pub struct SimplifiedArtist {
    pub external_urls: ExternalUrls,
    pub href: String,
    pub id: String,
    pub name: String,
    /// always "artist"
    pub r#type: String,
    pub uri: String,
}

/// https://developer.spotify.com/documentation/web-api/reference/get-information-about-the-users-current-playback
#[derive(Debug, Deserialize, Clone)]
pub struct Image {
    pub url: String,
    pub height: Option<u64>,
    pub width: Option<u64>,
}

/// https://developer.spotify.com/documentation/web-api/reference/get-information-about-the-users-current-playback
#[derive(Debug, Deserialize, Clone)]
pub struct Album {
    pub album_type: AlbumType,
    pub total_tracks: u64,
    pub external_urls: ExternalUrls,
    pub href: String,
    pub id: String,
    pub images: Vec<Image>,
    pub name: String,
    pub release_date: String,
    pub release_date_precision: ReleaseDatePrecision,
    pub restrictions: Option<Restrictions>,
    pub uri: String,
    pub artists: Vec<SimplifiedArtist>,
}

/// https://developer.spotify.com/documentation/web-api/reference/get-information-about-the-users-current-playback
#[derive(Debug, Deserialize, Clone)]
pub struct ExternalIds {
    /// International Standard Recoding Code
    pub isrc: Option<String>,
    /// International Article Number
    pub ean: Option<String>,
    /// Universal Product Code
    pub upc: Option<String>,
}

/// https://developer.spotify.com/documentation/web-api/reference/get-information-about-the-users-current-playback
#[derive(Debug, Deserialize, Clone)]
pub struct Track {
    pub album: Album,
    pub artists: Vec<SimplifiedArtist>,
    pub disc_number: u64,
    pub duration_ms: u64,
    pub explicit: bool,
    pub external_ids: HashMap<String, String>, // e.g. isrc: "JPP561100856"
    pub external_urls: ExternalUrls,
    pub href: String,
    pub id: String,
    pub is_local: bool,
    pub is_playable: Option<bool>,
    pub name: String,
    pub popularity: u64,
    pub preview_url: Option<String>,
    pub track_number: u64,
    pub uri: String,
}

/// https://developer.spotify.com/documentation/web-api/reference/get-information-about-the-users-current-playback
#[derive(Debug, Deserialize, Clone)]
pub struct EpisodeResumePoint {
    pub fully_played: bool,
    pub resume_position_ms: u64,
}

/// https://developer.spotify.com/documentation/web-api/reference/get-information-about-the-users-current-playback
#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum Copyright {
    #[serde(rename = "C")]
    Copyright { text: String },
    #[serde(rename = "P")]
    PerformanceCopyright { text: String },
}

/// https://developer.spotify.com/documentation/web-api/reference/get-information-about-the-users-current-playback
#[derive(Debug, Deserialize, Clone)]
pub struct Show {
    pub copyrights: Vec<Copyright>,
    pub description: String,
    pub html_description: String,
    pub explicit: bool,
    pub external_urls: ExternalUrls,
    pub href: String,
    pub id: String,
    pub images: Vec<Image>,
    pub is_externally_hosted: bool,
    /// ISO639
    pub languages: Vec<String>,
    pub media_type: String,
    pub name: String,
    pub publisher: String,
    pub uri: String,
    pub total_episodes: u64,
}


/// https://developer.spotify.com/documentation/web-api/reference/get-information-about-the-users-current-playback
#[derive(Debug, Deserialize, Clone)]
pub struct Episode {
    pub description: String,
    pub html_description: String,
    pub duration_ms: u64,
    pub explicit: bool,
    pub external_urls: ExternalUrls,
    pub href: String,
    pub id: String,
    pub images: Vec<Image>,
    pub is_externally_hosted: bool,
    pub is_playable: Option<bool>,
    /// ISO639
    pub languages: Vec<String>,
    pub name: String,
    pub release_date: String,
    pub release_date_precision: ReleaseDatePrecision,
    pub resume_point: EpisodeResumePoint,
    pub uri: String,
    pub restrictions: Option<Restrictions>,
    pub show: Show,
}


#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type")]
pub enum Item {
    Track(Track),
    Episode(Episode),
}

/// https://developer.spotify.com/documentation/web-api/reference/get-information-about-the-users-current-playback
#[derive(Deserialize_enum_str, Debug, PartialEq, Eq, Clone, Copy)]
pub enum PlaybackStateContextType {
    Artist,
    Playlist,
    Album,
    Show,
}

/// https://developer.spotify.com/documentation/web-api/reference/get-information-about-the-users-current-playback
#[derive(Debug, Deserialize, Clone)]
pub struct PlaybackStateContext {
    pub external_urls: ExternalUrls,
    pub href: String,
    pub r#type: String,
    pub uri: String,
}

/// reverse engineered struct
#[derive(Debug, Deserialize, Clone)]
pub struct PlaybackStateActionDisalllows {
    pub interrupting_playback: Option<bool>,
    pub pausing: Option<bool>,
    pub resuming: Option<bool>,
    pub seeking: Option<bool>,
    pub skipping_next: Option<bool>,
    pub skipping_prev: Option<bool>,
    pub toggling_repeat_context: Option<bool>,
    pub toggling_shuffle: Option<bool>,
    pub toggling_repeat_track: Option<bool>,
    pub transferring_playback: Option<bool>,
}

/// https://developer.spotify.com/documentation/web-api/reference/get-information-about-the-users-current-playback
#[derive(Debug, Deserialize, Clone)]
pub struct PlaybackStateActions {
    pub interrupting_playback: Option<bool>,
    pub pausing: Option<bool>,
    pub resuming: Option<bool>,
    pub seeking: Option<bool>,
    pub skipping_next: Option<bool>,
    pub skipping_prev: Option<bool>,
    pub toggling_repeat_context: Option<bool>,
    pub toggling_shuffle: Option<bool>,
    pub toggling_repeat_track: Option<bool>,
    pub transferring_playback: Option<bool>,

    /// reverse engineered field
    pub disallows: PlaybackStateActionDisalllows,
}

/// https://developer.spotify.com/documentation/web-api/reference/get-information-about-the-users-current-playback
#[derive(Debug, Deserialize, Clone)]
pub struct PlaybackState {
    pub device: Device,
    pub repeat_state: RepeatState,
    pub shuffle_state: bool,
    pub context: Option<PlaybackStateContext>,
    pub timestamp: u64,
    pub progress_ms: Option<u64>,
    pub is_playing: bool,
    pub item: Option<Item>,
    pub currently_playing_type: CurrentlyPlayingType,
    pub actions: PlaybackStateActions,
}

/// Decode structures for PLAYER_STATE_CHANGED
#[derive(Debug, Deserialize, Clone)]
pub struct PlayerStateChanged {
    pub state: PlaybackState,
}
