use eolib::protocol::{
    net::{server::LockerBuyServerPacket, PacketAction, PacketFamily},
    r#pub::NpcType,
};

use crate::{NPC_DB, SETTINGS};

use super::super::Map;

impl Map {
    pub fn upgrade_locker(&mut self, player_id: i32, npc_index: i32) {
        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        if character.bank_level >= SETTINGS.bank.max_upgrades {
            return;
        }

        let cost = SETTINGS.bank.upgrade_base_cost
            + SETTINGS.bank.upgrade_cost_step * character.bank_level;

        if character.get_item_amount(1) < cost {
            return;
        }

        let npc = match self.npcs.get(&npc_index) {
            Some(npc) => npc,
            None => return,
        };

        let npc_data = match NPC_DB.npcs.get(npc.id as usize - 1) {
            Some(npc_data) => npc_data,
            None => return,
        };

        if npc_data.r#type != NpcType::Bank {
            return;
        }

        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => return,
        };

        character.remove_item(1, cost);
        character.bank_level += 1;

        if let Some(player) = character.player.as_ref() {
            player.send(
                PacketAction::Buy,
                PacketFamily::Locker,
                &LockerBuyServerPacket {
                    gold_amount: character.get_item_amount(1),
                    locker_upgrades: character.bank_level,
                },
            );
        }
    }
}
