use specs::prelude::*;
use specs_derive::*;
use std::cmp:: {max, min};
use serde::{Serialize, Deserialize};

use super:: {Position, Viewshed, Map, CombatStats, Point, WantsToMelee,
			Item, GameLog, WantsToPickupItem, WantsToUseItem, DropItem,
			Name, InBackpack, RunState, TileType,
			Ranged, MAPSIZE_HEIGHT, MAPSIZE_WIDTH};

#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Player {}

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) -> RunState {
	let mut positions = ecs.write_storage::<Position>();
	let mut players = ecs.write_storage::<Player>();
	let mut viewsheds = ecs.write_storage::<Viewshed>();
	let combat_states = ecs.read_storage::<CombatStats>();
	let mut want_to_melee = ecs.write_storage::<WantsToMelee>();
	let entities = ecs.entities();
	let map = ecs.fetch::<Map>();

	for (entity, _player, pos, viewshed) in (&entities, &mut players, &mut positions, &mut viewsheds).join() {
		if pos.x + delta_x < 1 || pos.x + delta_x > map.width - 1 ||  
			pos.y + delta_y < 1 || pos.y + delta_y > map.height - 1 { 
				return RunState::AwaitingInput
			} 
		let destination_idx = map.xy_idx(pos.x + delta_x, pos.y + delta_y);

		for potential_target in map.tile_content[destination_idx].iter() {
			let target = combat_states.get(*potential_target);
			match target {
				None => {}
				Some(t) => {
					want_to_melee.insert(entity, WantsToMelee{ target: *potential_target }).expect("Add target failed");
					return RunState::PlayerTurn
				}
			}
		}


		if !map.blocked[destination_idx] {

			pos.x = min(MAPSIZE_WIDTH as i32, max(0, pos.x + delta_x));
			pos.y = min(MAPSIZE_HEIGHT as i32, max(0, pos.y + delta_y));

			let mut ppos = ecs.write_resource::<Point>();
			ppos.x = pos.x;
			ppos.y = pos.y;

			viewshed.dirty = true;

			return RunState::PlayerTurn
		}
	}
	RunState::PlayerTurn

}

pub fn try_next_level(ecs: &mut World) -> RunState {
	let player_pos = ecs.fetch::<Point>();
	let map = ecs.fetch::<Map>();
	let player_idx = map.xy_idx(player_pos.x, player_pos.y);
	if map.tiles[player_idx].tiletype == TileType::DownStairs{
		RunState::NextLevel
	} else {
		let mut gamelog = ecs.fetch_mut::<GameLog>();
		gamelog.entries.push(GameLog::cannot_down_log());
		RunState::AwaitingInput
	}
}

pub fn get_item(ecs: &mut World) -> RunState {
    let player_pos = ecs.fetch::<Point>();
    let player_entity = ecs.fetch::<Entity>();
    let entities = ecs.entities();
    let items = ecs.read_storage::<Item>();
    let positions = ecs.read_storage::<Position>();
    let mut gamelog = ecs.fetch_mut::<GameLog>();

    let mut target_item : Option<Entity> = None;
    for (item_entity, _item, position) in (&entities, &items, &positions).join() {
        if position.x == player_pos.x && position.y == player_pos.y {
            target_item = Some(item_entity);
        }
    }

    match target_item {
        None => gamelog.entries.push(GameLog::try_get_but_nothing_log()),
        Some(item) => {
            let mut pickup = ecs.write_storage::<WantsToPickupItem>();
            pickup.insert(*player_entity, WantsToPickupItem{ collected_by: *player_entity, item }).expect("Unable to insert want to pickup");
        }
    }
	RunState::PlayerTurn
}

pub fn try_use_item(ecs: &mut World, keynum: i32) -> RunState {
	let player_entity = ecs.fetch::<Entity>();
	let names = ecs.read_storage::<Name>();
	let backpack = ecs.read_storage::<InBackpack>();
	let mut gamelog = ecs.fetch_mut::<GameLog>();
	let entities = ecs.entities();

	let mut j = 0;
	for (entity, _pack, name) in (&entities, &backpack, &names).join().filter(|item| item.1.owner == *player_entity) {
		if j == keynum {
			let is_ranged = ecs.read_storage::<Ranged>();
			let item_raged = is_ranged.get(entity);
			match item_raged {
				None => {
					let mut intent = ecs.write_storage::<WantsToUseItem>();
					intent.insert(*ecs.fetch::<Entity>(), WantsToUseItem {item: entity, target: None})
						.expect("Cannot WantsToUseItem");
					return RunState::PlayerTurn;
				}
				Some(item_raged) => {
					return RunState::ShowTargeting {
						range: item_raged.range,
						item: entity
					}
				}
			}
		}		
		j += 1;
	}
	gamelog.entries.push(GameLog::try_do_item_but_no_item());
	RunState::AwaitingInput
}

pub fn try_drop_item(ecs: &mut World, keynum: i32) -> RunState {
	let player_entity = ecs.fetch::<Entity>();
	let names = ecs.read_storage::<Name>();
	let backpack = ecs.read_storage::<InBackpack>();
	let mut gamelog = ecs.fetch_mut::<GameLog>();
	let entities = ecs.entities();

	let mut j = 0;
	for (entity, _pack, name) in (&entities, &backpack, &names).join().filter(|item| item.1.owner == *player_entity) {
		if j == keynum {
			let mut intent = ecs.write_storage::<DropItem>();
			intent.insert(*ecs.fetch::<Entity>(), DropItem { item: entity })
				.expect("Cannot DropItem");
			return RunState::PlayerTurn;
		}		
		j += 1;
	}
	gamelog.entries.push(GameLog::try_do_item_but_no_item());
	RunState::AwaitingInput
}