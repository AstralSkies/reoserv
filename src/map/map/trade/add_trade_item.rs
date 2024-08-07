use eolib::protocol::{net::Item, r#pub::ItemSpecial};

use crate::{ITEM_DB, SETTINGS};

use super::super::Map;

const MAX_TRADE_SLOTS: usize = 10;

impl Map {
    pub fn add_trade_item(&mut self, player_id: i32, partner_id: i32, item: Item) {
        if item.amount <= 0
            || item.amount > SETTINGS.limits.max_trade
            || SETTINGS.items.protected_items.contains(&item.id)
        {
            return;
        }

        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => return,
        };

        if character.get_item_amount(item.id) < item.amount {
            return;
        }

        let offered = character.trade_items.iter().any(|i| i.id == item.id);

        if !offered && character.trade_items.len() >= MAX_TRADE_SLOTS {
            return;
        }

        let item_data = match ITEM_DB.items.get(item.id as usize - 1) {
            Some(item_data) => item_data,
            None => return,
        };

        if item_data.special == ItemSpecial::Lore {
            return;
        }

        if offered {
            let trade_item = character
                .trade_items
                .iter_mut()
                .find(|i| i.id == item.id)
                .unwrap();

            trade_item.amount = item.amount;
        } else {
            character.trade_items.push(item);
        }

        self.send_trade_update(player_id, partner_id);
    }
}
