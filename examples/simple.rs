use spotnowplay::*;

struct App;

#[async_trait::async_trait]
impl EventHandler for App {
    async fn on_playback_state_update(&self, player_state: Option<&api::PlaybackState>) {
        let Some(player_state) = player_state else {
            println!("Inactive");
            return;
        };

        let name = match player_state.item.as_ref().unwrap() {
            api::Item::Track(track) => &track.name,
            api::Item::Episode(episode) => &episode.name,
        };

        if player_state.is_playing {
            println!("♪ {name}");
        } else {
            println!("Paused");
        }
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let token = discord_integration::get_available_spotify_token(
        &std::env::var("DISCORD_TOKEN")
            .expect("Specified the Discord token at DISCORD_TOKEN environment"),
    )
    .await
    .unwrap();

    let client = ClientBuilder::new(&token.access_token)
        .handler(App {})
        .build();

    client.run().await.unwrap();
}
