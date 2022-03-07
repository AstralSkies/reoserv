use eo::{
    data::{Serializeable, StreamReader},
    net::{
        packets::{
            client::welcome::Message,
        },
        Action, Family,
    },
};

use crate::{
    player::PlayerHandle,
    PacketBuf, world::WorldHandle,
};

pub async fn message(
    buf: PacketBuf,
    player: PlayerHandle,
    world: WorldHandle,
) {
    let mut request = Message::default();
    let reader = StreamReader::new(&buf);
    request.deserialize(&reader);

    debug!("Recv: {:?}", request);

    match world.enter_game(player.clone()).await {
        Ok(reply) => {
            debug!("Reply: {:?}", reply);

            player.send(
                Action::Reply,
                Family::Welcome,
                reply.serialize(),
            );
        },
        Err(_) => {}
    }

    // let mut reply = Reply::new();
    // reply.reply = WelcomeReply::EnterGame;

    // let characters = characters.lock().await;
    // let player_character = characters
    //     .iter()
    //     .find(|c| c.player_id == player_id)
    //     .unwrap();
    // let nearby_characters: Vec<&Character> = characters
    //     .iter()
    //     .filter(|c| {
    //         c.id == player_character.id
    //             || c.map_id == player_character.map_id
    //                 && in_range(
    //                     player_character.coords.x as f64,
    //                     player_character.coords.y as f64,
    //                     c.coords.x as f64,
    //                     c.coords.y as f64,
    //                 )
    //     })
    //     .collect();

    // let mut enter_game = EnterGame::new();
    // enter_game.news[0] = "Welcome to my server! Powered by reoserv.".to_string();
    // enter_game.news[1] =
    //     "[Feb 18] Players can enter the game world but are still alone!".to_string();
    // enter_game.weight = Weight {
    //     current: 0,
    //     max: 70,
    // };
    // enter_game.items = vec![];
    // enter_game.spells = vec![];
    // enter_game.nearby_info = NearbyInfo::default();
    // enter_game.nearby_info.characters = nearby_characters.iter().map(|character| CharacterMapInfo {
    //     name: character.name.to_string(),
    //     id: character.player_id,
    //     map_id: character.map_id,
    //     coords: character.coords,
    //     direction: character.direction,
    //     class_id: character.class,
    //     guild_tag: String::default(),
    //     level: character.level,
    //     gender: character.gender,
    //     hair_style: character.hair_style as EOChar,
    //     hair_color: character.hair_color as EOChar,
    //     race: character.race,
    //     max_hp: character.max_hp,
    //     hp: character.hp,
    //     max_tp: character.max_tp,
    //     tp: character.tp,
    //     paperdoll: PaperdollB000A0HSW {
    //         boots: character.paperdoll.boots,
    //         armor: character.paperdoll.armor,
    //         hat: character.paperdoll.hat,
    //         shield: character.paperdoll.shield,
    //         weapon: character.paperdoll.weapon,
    //     },
    //     sit_state: character.sit_state,
    //     invisible: character.hidden,
    // }).collect();
    // reply.enter_game = Some(enter_game);

    // debug!("Reply: {:?}", reply);

    // tx.send(PlayerCommand::SetState(State::Playing(request.character_id)))?;
    // tx.send(PlayerCommand::Send(
    //     Action::Reply,
    //     Family::Welcome,
    //     reply.serialize(),
    // ))?;
}
