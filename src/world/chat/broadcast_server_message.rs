use std::collections::HashMap;

use eo::{
    data::{EOShort, Serializeable},
    net::{packets::server::talk, Action, Family},
};

use crate::player::{PlayerHandle, State};

pub async fn broadcast_server_message(message: &str, players: &HashMap<EOShort, PlayerHandle>) {
    let packet = talk::Server {
        message: message.to_string(),
    };
    let buf = packet.serialize();
    for player in players.values() {
        let state = player.get_state().await;
        if state == State::Playing {
            player.send(Action::Server, Family::Talk, buf.clone());
        }
    }
}