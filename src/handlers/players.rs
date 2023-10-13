use eo::{
    data::{Serializeable, StreamBuilder, StreamReader},
    protocol::{
        server::init::{Init, InitData, InitFriendListPlayers, InitPlayers},
        InitReply, PacketAction, PacketFamily,
    },
};

use crate::{player::PlayerHandle, world::WorldHandle};

pub async fn request(player: PlayerHandle, world: WorldHandle) {
    let players = world.get_online_list().await;

    let reply = Init {
        reply_code: InitReply::Players,
        data: InitData::Players(InitPlayers {
            num_online: players.len() as u16,
            list: players,
        }),
    };

    let mut builder = StreamBuilder::new();
    reply.serialize(&mut builder);
    player.send(PacketAction::Init, PacketFamily::Init, builder.get());
}

pub async fn list(player: PlayerHandle, world: WorldHandle) {
    let players = world.get_online_list().await;

    let reply = Init {
        reply_code: InitReply::FriendListPlayers,
        data: InitData::FriendListPlayers(InitFriendListPlayers {
            num_online: players.len() as u16,
            list: players.iter().map(|p| p.name.to_owned()).collect(),
        }),
    };

    let mut builder = StreamBuilder::new();
    reply.serialize(&mut builder);
    player.send(PacketAction::Init, PacketFamily::Init, builder.get());
}

pub async fn players(
    action: PacketAction,
    _reader: StreamReader,
    player: PlayerHandle,
    world: WorldHandle,
) {
    match action {
        PacketAction::Request => request(player, world).await,
        PacketAction::List => list(player, world).await,
        _ => error!("Unhandled packet Players_{:?}", action),
    }
}
