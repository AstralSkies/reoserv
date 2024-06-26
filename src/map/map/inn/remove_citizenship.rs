use eolib::protocol::{
    net::{
        server::{CitizenRemoveServerPacket, InnUnsubscribeReply},
        PacketAction, PacketFamily,
    },
    r#pub::NpcType,
};

use crate::{INN_DB, NPC_DB, SETTINGS};

use super::super::Map;

impl Map {
    pub fn remove_citizenship(&mut self, player_id: i32, npc_index: i32) {
        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => return,
        };

        let player = match character.player.as_ref() {
            Some(player) => player,
            None => return,
        };

        let npc = match self.npcs.get(&npc_index) {
            Some(npc) => npc,
            None => return,
        };

        let npc_data = match NPC_DB.npcs.get(npc.id as usize - 1) {
            Some(npc_data) => npc_data,
            None => return,
        };

        if npc_data.r#type != NpcType::Inn {
            return;
        }

        let inn_data = match INN_DB
            .inns
            .iter()
            .find(|inn| inn.behavior_id == npc_data.behavior_id)
        {
            Some(inn_data) => inn_data,
            None => return,
        };

        player.send(
            PacketAction::Remove,
            PacketFamily::Citizen,
            &CitizenRemoveServerPacket {
                reply_code: if character.home == SETTINGS.new_character.home
                    || character.home != inn_data.name
                {
                    InnUnsubscribeReply::NotCitizen
                } else {
                    character.home = SETTINGS.new_character.home.to_owned();
                    InnUnsubscribeReply::Unsubscribed
                },
            },
        );
    }
}
