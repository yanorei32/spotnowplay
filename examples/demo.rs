use std::cell::Cell;
use std::io::Write;
use std::sync::Mutex;

use spotnowplay::*;

struct App {
    update_task: Mutex<Cell<Option<tokio::task::JoinHandle<()>>>>,
    last_played: Mutex<String>,
}

fn draw_progress(playing: bool, current: u64, total: u64) {
    if playing {
        print!("\r\x1b[KPlaying ");
    } else {
        print!("\r\x1b[KPaused  ");
    }

    let total = total.max(1);
    let count = ((current * 70 + total - 1) / total) as usize;

    print!("[{}{}]", "=".repeat(count), " ".repeat(70 - count));
    std::io::stdout().flush().unwrap();
}

#[async_trait::async_trait]
impl EventHandler for App {
    async fn on_playback_state_update(&self, player_state: Option<&api::PlaybackState>) {
        let update_task = (*self.update_task.lock().unwrap()).replace(None);

        if let Some(update_task) = update_task {
            update_task.abort();
        }

        let Some(player_state) = player_state else {
            println!("\r\n\r\nInactive");
            return;
        };

        let item_id = match player_state.item.as_ref().unwrap() {
            api::Item::Track(track) => track.uri.clone(),
            api::Item::Episode(episode) => episode.uri.clone(),
        };

        let need_rewrite_item_information = &*self.last_played.lock().unwrap() != &item_id;
        *self.last_played.lock().unwrap() = item_id.clone();

        if need_rewrite_item_information {
            println!();
            println!();

            match player_state.item.as_ref().unwrap() {
                api::Item::Track(track) => {
                    println!("Album:  {}", track.album.name);

                    for artist in &track.artists {
                        println!("Artist: {}", artist.name);
                    }

                    println!("Title:  {}", track.name);
                }
                api::Item::Episode(episode) => {
                    println!("Title:  {}", episode.name);
                }
            };
        }

        let duration_ms = match player_state.item.as_ref().unwrap() {
            api::Item::Track(track) => track.duration_ms,
            api::Item::Episode(episode) => episode.duration_ms,
        };

        let progress_ms = player_state.progress_ms.unwrap();

        if player_state.is_playing {
            let update_task = tokio::spawn(async move {
                let instant = std::time::Instant::now();
                loop {
                    draw_progress(
                        true,
                        progress_ms + instant.elapsed().as_millis() as u64,
                        duration_ms,
                    );
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                }
            });

            (*self.update_task.lock().unwrap()).replace(Some(update_task));
        } else {
            draw_progress(false, progress_ms, duration_ms);
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
        .handler(App {
            update_task: Mutex::new(Cell::new(None)),
            last_played: Mutex::new(String::from("")),
        })
        .build();

    client.run().await.unwrap();
}
