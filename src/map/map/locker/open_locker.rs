use eolib::protocol::{
    map::MapTileSpec,
    net::{server::LockerOpenServerPacket, PacketAction, PacketFamily, ThreeItem},
    Coords,
};

use super::super::Map;

impl Map {
    pub fn open_locker(&self, player_id: i32) {
        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        let player = match character.player.as_ref() {
            Some(player) => player,
            None => return,
        };

        let adjacent_tiles = [
            self.get_tile(&Coords {
                x: character.coords.x,
                y: character.coords.y - 1,
            }),
            self.get_tile(&Coords {
                x: character.coords.x,
                y: character.coords.y + 1,
            }),
            self.get_tile(&Coords {
                x: character.coords.x - 1,
                y: character.coords.y,
            }),
            self.get_tile(&Coords {
                x: character.coords.x + 1,
                y: character.coords.y,
            }),
        ];

        if !adjacent_tiles.iter().any(|tile| match tile {
            Some(tile) => *tile == MapTileSpec::BankVault,
            None => false,
        }) {
            return;
        }

        player.send(
            PacketAction::Open,
            PacketFamily::Locker,
            &LockerOpenServerPacket {
                locker_coords: character.coords,
                locker_items: character
                    .bank
                    .iter()
                    .map(|item| ThreeItem {
                        id: item.id,
                        amount: item.amount,
                    })
                    .collect(),
            },
        );
    }
}
