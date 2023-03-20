use eo::{
    data::{EOShort, Serializeable},
    protocol::{server::door, Coords, PacketAction, PacketFamily},
};

use super::Map;

impl Map {
    pub fn open_door(&self, target_player_id: EOShort, door_coords: Coords) {
        let target = self.characters.get(&target_player_id).unwrap();
        if target.is_in_range(door_coords) {
            let packet = door::Open {
                coords: door_coords,
            };
            let buf = packet.serialize();
            for character in self.characters.values() {
                if character.is_in_range(door_coords) {
                    character.player.as_ref().unwrap().send(
                        PacketAction::Open,
                        PacketFamily::Door,
                        buf.clone(),
                    );
                }
            }
        }
    }
}