use eo::{
    data::{EOInt, EOShort, StreamReader, MAX1},
    net::{Action, Family},
};
use num_traits::FromPrimitive;

use super::{handlers, PlayerHandle};

use crate::{world::WorldHandle, PacketBuf, SETTINGS};

pub async fn handle_packet(
    packet: PacketBuf,
    player_id: EOShort,
    player: PlayerHandle,
    world: WorldHandle,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let action = Action::from_u8(packet[0]).unwrap();
    let family = Family::from_u8(packet[1]).unwrap();
    let reader = StreamReader::new(&packet[2..]);

    if family != Family::Init {
        if family == Family::Connection && action == Action::Ping {
            player.pong_new_sequence().await;
        }

        let server_sequence = player.gen_sequence().await;
        let client_sequence = if server_sequence >= MAX1 {
            reader.get_short() as EOInt
        } else {
            reader.get_char() as EOInt
        };

        if SETTINGS.server.enforce_sequence && server_sequence != client_sequence {
            player.close(format!(
                "sending invalid sequence: Got {}, expected {}.",
                client_sequence, server_sequence
            ));
        }
    } else {
        let _ = player.gen_sequence().await;
    }

    let buf = reader.get_vec(reader.remaining());
    match family {
        Family::Init => match action {
            Action::Init => {
                handlers::init::request(buf, player_id, player.clone()).await;
            }
            _ => {
                error!("Unhandled packet {:?}_{:?}", action, family);
            }
        },
        Family::Connection => match action {
            Action::Accept => {
                handlers::connection::accept(buf, player_id, player.clone()).await;
            }
            Action::Ping => {
                player.pong();
            }
            _ => {
                error!("Unhandled packet {:?}_{:?}", action, family);
            }
        },
        Family::Account => match action {
            Action::Request => {
                handlers::account::request(buf, player.clone(), world.clone()).await;
            }
            Action::Create => {
                handlers::account::create(buf, player.clone(), world.clone()).await;
            }
            _ => {
                error!("Unhandled packet {:?}_{:?}", action, family);
            }
        },
        Family::Login => match action {
            Action::Request => {
                handlers::login::request(buf, player.clone(), world.clone()).await?;
            }
            _ => {
                error!("Unhandled packet {:?}_{:?}", action, family);
            }
        },
        Family::Character => match action {
            Action::Request => {
                handlers::character::request(
                    buf,
                    player.clone(),
                    world.clone()
                )
                .await;
            }
            Action::Create => {
                handlers::character::create(
                    buf,
                    player.clone(),
                    world.clone()
                )
                .await;
            }
            Action::Take => {
                handlers::character::take(
                    buf,
                    player.clone(),
                    world.clone()
                )
                .await;
            }
            Action::Remove => {
                handlers::character::remove(
                    buf,
                    player.clone(),
                    world.clone()
                )
                .await;
            }
            _ => {
                error!("Unhandled packet {:?}_{:?}", action, family);
            }
        },
        //     Family::Welcome => match action {
        //         Action::Request => {
        //             let mut conn = db_pool.get_conn().await?;
        //             handlers::welcome::request(
        //                 buf,
        //                 players.lock().await.get(&player.id).unwrap(),
        //                 &mut conn,
        //                 world,
        //                 player.id,
        //                 characters.clone(),
        //             )
        //             .await?;
        //         }
        //         Action::Agree => {
        //             let characters = characters.lock().await;
        //             let character = characters
        //                 .iter()
        //                 .find(|c| c.player_id == player.id)
        //                 .unwrap();
        //             handlers::welcome::agree(
        //                 buf,
        //                 players.lock().await.get(&player.id).unwrap(),
        //                 world,
        //                 character.map_id,
        //             )
        //             .await?;
        //         }
        //         Action::Message => {
        //             handlers::welcome::message(
        //                 buf,
        //                 players.lock().await.get(&player.id).unwrap(),
        //                 characters.clone(),
        //                 player.id,
        //             )
        //             .await?;
        //         }
        //         _ => {
        //             error!("Unhandled packet {:?}_{:?}", action, family);
        //         }
        //     },
        _ => {
            error!("Unhandled packet {:?}_{:?}", action, family);
        }
    }

    player.set_busy(false);

    Ok(())
}