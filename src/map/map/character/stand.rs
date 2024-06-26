use eolib::protocol::net::{
    server::{ChairCloseServerPacket, SitCloseServerPacket, SitState},
    PacketAction, PacketFamily,
};

use crate::utils::get_next_coords;

use super::super::Map;

impl Map {
    pub fn stand(&mut self, player_id: i32) {
        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => {
                error!("Failed to get character {}", player_id);
                return;
            }
        };

        match character.sit_state {
            SitState::Floor => {
                character.sit_state = SitState::Stand;

                let reply = SitCloseServerPacket {
                    player_id,
                    coords: character.coords,
                };

                if let Some(player) = character.player.as_ref() {
                    player.send(PacketAction::Close, PacketFamily::Sit, &reply);
                }

                if !character.hidden {
                    self.send_packet_near_player(
                        player_id,
                        PacketAction::Remove,
                        PacketFamily::Sit,
                        &reply,
                    );
                }
            }
            SitState::Chair => {
                character.sit_state = SitState::Stand;

                character.coords = get_next_coords(
                    &character.coords,
                    character.direction,
                    self.file.width,
                    self.file.height,
                );

                let packet = ChairCloseServerPacket {
                    player_id,
                    coords: character.coords,
                };

                if let Some(player) = character.player.as_ref() {
                    player.send(PacketAction::Close, PacketFamily::Chair, &packet);
                }

                if !character.hidden {
                    self.send_packet_near_player(
                        player_id,
                        PacketAction::Remove,
                        PacketFamily::Sit,
                        &packet,
                    );
                }
            }
            _ => {}
        }
    }
}
