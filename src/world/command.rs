use eo::{
    data::{EOShort, EOInt},
    net::packets::server::{account, character, login},
};
use tokio::sync::oneshot;

use crate::player::PlayerHandle;

#[derive(Debug)]
pub enum Command {
    LoadPubFiles {
        respond_to: oneshot::Sender<()>,
    },
    LoadMapFiles {
        respond_to: oneshot::Sender<()>,
    },
    StartPingTimer {
        respond_to: oneshot::Sender<()>,
    },
    GetPlayerCount {
        respond_to: oneshot::Sender<usize>,
    },
    GetNextPlayerId {
        respond_to: oneshot::Sender<EOShort>,
    },
    AddPlayer {
        respond_to: oneshot::Sender<()>,
        player_id: EOShort,
        player: PlayerHandle,
    },
    DropPlayer {
        player_id: EOShort,
        account_id: EOShort,
        character_id: EOShort,
        respond_to: oneshot::Sender<()>,
    },
    RequestAccountCreation {
        name: String,
        player: PlayerHandle,
        respond_to:
            oneshot::Sender<Result<account::Reply, Box<dyn std::error::Error + Send + Sync>>>,
    },
    CreateAccount {
        details: eo::net::packets::client::account::Create,
        register_ip: String,
        respond_to:
            oneshot::Sender<Result<account::Reply, Box<dyn std::error::Error + Send + Sync>>>,
    },
    Login {
        name: String,
        password: String,
        respond_to: oneshot::Sender<
            Result<(login::Reply, EOShort), Box<dyn std::error::Error + Send + Sync>>,
        >,
    },
    RequestCharacterCreation {
        player: PlayerHandle,
        respond_to:
            oneshot::Sender<Result<character::Reply, Box<dyn std::error::Error + Send + Sync>>>,
    },
    CreateCharacter {
        details: eo::net::packets::client::character::Create,
        player: PlayerHandle,
        respond_to:
            oneshot::Sender<Result<character::Reply, Box<dyn std::error::Error + Send + Sync>>>,
    },
    RequestCharacterDeletion {
        character_id: EOInt,
        player: PlayerHandle,
        respond_to:
            oneshot::Sender<Result<character::Player, Box<dyn std::error::Error + Send + Sync>>>,
    },
    DeleteCharacter {
        session_id: EOShort,
        character_id: EOInt,
        player: PlayerHandle,
        respond_to:
            oneshot::Sender<Result<character::Reply, Box<dyn std::error::Error + Send + Sync>>>,
    }
}