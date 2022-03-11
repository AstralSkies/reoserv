use std::{io::Cursor, path::Path};

use eo::{net::{
    packets::server::welcome::{EnterGame, Reply},
    replies::WelcomeReply, Weight,
}, data::EOChar};

use crate::{map::MapHandle, player::PlayerHandle};

use tokio::io::{AsyncBufReadExt, AsyncReadExt};

pub async fn enter_game(
    map: MapHandle,
    player: PlayerHandle,
) -> Result<Reply, Box<dyn std::error::Error + Send + Sync>> {
    let player_id = player.get_player_id().await;
    player.set_map(map.clone());
    let mut character = player.take_character().await?;
    character.calculate_stats().await;

    let weight = Weight {
        current: character.weight as EOChar,
        max: character.max_weight as EOChar,
    };
    let items = character.items.clone();
    let spells = character.spells.clone();

    map.enter(character).await;
    let nearby_info = map.get_nearby_info(player_id).await;
    Ok(Reply {
        reply: WelcomeReply::EnterGame,
        select_character: None,
        enter_game: Some(EnterGame {
            news: get_news().await,
            weight,
            items,
            spells,
            nearby_info,
        }),
    })
}

async fn get_news() -> [String; 9] {
    match tokio::fs::File::open(Path::new("news.txt")).await {
        Ok(mut file) => {
            let mut buf: Vec<u8> = Vec::new();
            file.read_to_end(&mut buf).await.unwrap();
            let cursor = Cursor::new(buf);
            let mut lines = cursor.lines();
            let mut news = [
                String::default(),
                String::default(),
                String::default(),
                String::default(),
                String::default(),
                String::default(),
                String::default(),
                String::default(),
                String::default(),
            ];
            for i in 0..9 {
                if let Ok(Some(line)) = lines.next_line().await {
                    news[i] = line
                }
            }
            news
        }
        Err(e) => {
            error!("Failed to open news.txt: {}", e);
            [
                String::default(),
                String::default(),
                String::default(),
                String::default(),
                String::default(),
                String::default(),
                String::default(),
                String::default(),
                String::default(),
            ]
        }
    }
}
