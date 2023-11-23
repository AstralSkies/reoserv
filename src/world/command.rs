use eo::{
    data::{EOChar, EOInt, EOShort},
    protocol::{client, FileType, OnlinePlayers},
};
use tokio::sync::oneshot;

use crate::{character::Character, map::MapHandle, player::PlayerHandle};

use super::{Party, WorldHandle};

#[derive(Debug)]
pub enum Command {
    AcceptPartyRequest {
        player_id: EOShort,
        target_player_id: EOShort,
        request_type: EOChar,
    },
    AddLoggedInAccount {
        account_id: EOInt,
    },
    AddPlayer {
        respond_to: oneshot::Sender<()>,
        player_id: EOShort,
        player: PlayerHandle,
    },
    BroadcastAdminMessage {
        name: String,
        message: String,
    },
    BroadcastAnnouncement {
        name: String,
        message: String,
    },
    BroadcastGlobalMessage {
        target_player_id: EOShort,
        name: String,
        message: String,
    },
    _BroadcastServerMessage {
        message: String,
    },
    BroadcastPartyMessage {
        player_id: EOShort,
        message: String,
    },
    ChangePassword {
        player_id: EOShort,
        username: String,
        current_password: String,
        new_password: String,
    },
    CreateAccount {
        player_id: EOShort,
        details: client::account::Create,
    },
    CreateCharacter {
        player_id: EOShort,
        details: client::character::Create,
    },
    DeleteCharacter {
        player_id: EOShort,
        session_id: EOShort,
        character_id: EOInt,
    },
    DropPlayer {
        player_id: EOShort,
        account_id: EOInt,
        character_name: String,
        respond_to: oneshot::Sender<()>,
    },
    EnterGame {
        player_id: EOShort,
        session_id: EOShort,
    },
    GetCharacterByName {
        name: String,
        respond_to:
            oneshot::Sender<Result<Box<Character>, Box<dyn std::error::Error + Sync + Send>>>,
    },
    GetFile {
        player_id: EOShort,
        file_type: FileType,
        session_id: EOShort,
        file_id: Option<EOChar>,
        warp: bool,
    },
    GetMap {
        map_id: EOShort,
        respond_to: oneshot::Sender<Result<MapHandle, Box<dyn std::error::Error + Send + Sync>>>,
    },
    GetNextPlayerId {
        respond_to: oneshot::Sender<EOShort>,
    },
    GetOnlineList {
        respond_to: oneshot::Sender<Vec<OnlinePlayers>>,
    },
    GetPlayerCount {
        respond_to: oneshot::Sender<usize>,
    },
    IsLoggedIn {
        account_id: EOInt,
        respond_to: oneshot::Sender<bool>,
    },
    LoadMapFiles {
        world: WorldHandle,
        respond_to: oneshot::Sender<()>,
    },
    Login {
        player_id: EOShort,
        name: String,
        password: String,
    },
    GetPlayerParty {
        player_id: EOShort,
        respond_to: oneshot::Sender<Option<Party>>,
    },
    PingPlayers,
    ReportPlayer {
        player_id: EOShort,
        reportee_name: String,
        message: String,
    },
    RequestAccountCreation {
        player_id: EOShort,
        name: String,
    },
    RequestCharacterCreation {
        player_id: EOShort,
    },
    RequestCharacterDeletion {
        player_id: EOShort,
        character_id: EOInt,
    },
    RequestPartyList {
        player_id: EOShort,
    },
    RemovePartyMember {
        player_id: EOShort,
        target_player_id: EOShort,
    },
    Save,
    SelectCharacter {
        player_id: EOShort,
        character_id: EOInt,
    },
    SendAdminMessage {
        player_id: EOShort,
        message: String,
    },
    SendPrivateMessage {
        from: PlayerHandle,
        to: String,
        message: String,
    },
    Shutdown {
        respond_to: oneshot::Sender<()>,
    },
    Tick,
    UpdatePartyHP {
        player_id: EOShort,
        hp_percentage: EOChar,
    },
}
