use eolib::protocol::{
    net::{server::StatSkillJunkServerPacket, PacketAction, PacketFamily},
    r#pub::NpcType,
};

use crate::NPC_DB;

use super::super::Map;

impl Map {
    pub fn reset_character(&mut self, player_id: i32, npc_index: i32) {
        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
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

        if npc_data.r#type != NpcType::Trainer {
            return;
        }

        character.reset();

        if let Some(player) = character.player.as_ref() {
            player.send(
                PacketAction::Junk,
                PacketFamily::StatSkill,
                &StatSkillJunkServerPacket {
                    stats: character.get_character_stats_reset(),
                },
            );
        }
    }
}
