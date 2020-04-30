use specs::prelude::*;
use super::{Map, SufferDamage, InflictsDamage, AreaOfEffect,
WantsToPickupItem, Name, InBackpack, Position, gamelog::GameLog,
WantsToUseItem, DropItem, ProvidesHealing, Consumable, CombatStats, Paralyze};

pub struct ItemCollectionSystem {}

impl<'a> System<'a> for ItemCollectionSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( ReadExpect<'a, Entity>,
                        WriteExpect<'a, GameLog>,
                        WriteStorage<'a, WantsToPickupItem>,
                        WriteStorage<'a, Position>,
                        ReadStorage<'a, Name>,
                        WriteStorage<'a, InBackpack>
                      );

    fn run(&mut self, data : Self::SystemData) {
        let (player_entity, mut gamelog, mut wants_pickup, mut positions, names, mut backpack) = data;

        for pickup in wants_pickup.join() {
            positions.remove(pickup.item);
            backpack.insert(pickup.item, InBackpack{ owner: pickup.collected_by })
				.expect("Unable to insert backpack entry");

            if pickup.collected_by == *player_entity {
                let item_name = &names.get(pickup.item).unwrap().name;
				gamelog.entries.push(GameLog::get_item_log(&item_name));
            }
        }

        wants_pickup.clear();
    }
}

pub struct ItemUseSystem {}
impl<'a> System<'a> for ItemUseSystem {
	#[allow(clippy::type_complexity)]
    type SystemData = ( ReadExpect<'a, Entity>,
                        WriteExpect<'a, GameLog>,
                        Entities<'a>,
                        WriteStorage<'a, WantsToUseItem>,
                        ReadStorage<'a, Name>,
						ReadStorage<'a, Consumable>,
						ReadStorage<'a, ProvidesHealing>,
                        WriteStorage<'a, CombatStats>,
						ReadStorage<'a, InflictsDamage>,
						ReadStorage<'a, AreaOfEffect>,
						WriteStorage<'a, SufferDamage>,
						WriteStorage<'a, Paralyze>,
						ReadExpect<'a, Map>
                      );
	fn run (&mut self, data: Self::SystemData) {
        let (player_entity, 
			mut gamelog, entities, 
			mut wants_use, 
			names, 
			consumables,
			heal_items,
			mut combat_stats,
			inflict_damage,
			aoe,
			mut suffer_damage,
			mut paralyze,
			map) = data;

		for (entity, use_item) in (&entities, &wants_use).join() {
			let consumable = consumables.get(use_item.item);
			match consumable {
				None => {},
				Some(_) => {
					entities.delete(use_item.item).expect("Delefe failed");
				}
			}

			let mut targets : Vec<Entity> = Vec::new();
			match use_item.target {
				None => { targets.push( *player_entity ); }
				Some(target) => {
					let area_effect = aoe.get(use_item.item);
					match area_effect {
						None => {
							let idx = map.xy_idx(target.x, target.y);
							for mob in map.tile_content[idx].iter() {
								targets.push(*mob);
							}
						},
						Some(area_effect) => {
							let mut blast_tiles = bracket_lib::prelude::field_of_view(target, area_effect.radius, &*map);
							blast_tiles.retain(|p| p.x > 0 && p.x < map.width && p.y > 0 && p.y < map.height - 1);
							for tile_idx in blast_tiles.iter() {
								let idx = map.xy_idx(tile_idx.x, tile_idx.y);
								for mob in map.tile_content[idx].iter() {
									targets.push(*mob);
								}
							}
						}
					}
				}
			}

			let heal_item = heal_items.get(use_item.item);
			match heal_item {
				None => {},
				Some(healer) => {
					for target in targets.iter() {
						let stats = combat_stats.get_mut(*target);
						match stats {
							None => {}
							Some(stats) => {
								stats.hp = i32::min(stats.max_hp, stats.hp + healer.heal_amount);
								if entity == *player_entity {
										gamelog.entries.push(
											GameLog::heal_log(
											&names.get(use_item.item).unwrap().name,
											healer.heal_amount));
								}
							}
						}
					}
				}
			}

			let item_damages = inflict_damage.get(use_item.item);
			match item_damages {
				None => {}
				Some(damage) => {
					for mob in targets.iter(){
						SufferDamage::new_damage(&mut suffer_damage, *mob, damage.damage);
						if entity == *player_entity {
							let player_name = names.get(*player_entity).unwrap();
							let mob_name = names.get(*mob).unwrap();
							gamelog.entries.push(GameLog::battle_log(&player_name.name, &mob_name.name, damage.damage));
						}
					}
				}
			}

			let mut add_paralyze = Vec::new();
			{
				let causes_paralize = paralyze.get(use_item.item);
				match causes_paralize {
					None => {}
					Some(paralyze) => {
						for mob in targets.iter() {
							add_paralyze.push((*mob, paralyze.turns));
							let mob_name = names.get(*mob).unwrap();
							gamelog.entries.push(GameLog::paralyze_log(&mob_name.name));
						}
					}
				}

			}
			for mob in add_paralyze.iter() {
				paralyze.insert(mob.0 , Paralyze {turns: mob.1}).unwrap();
			}

		}
		wants_use.clear();
	}
}

pub struct ItemDropSystem {}
impl <'a> System<'a> for ItemDropSystem {

    #[allow(clippy::type_complexity)]
    type SystemData = ( ReadExpect<'a, Entity>,
                        WriteExpect<'a, GameLog>,
                        Entities<'a>,
                        WriteStorage<'a, DropItem>,
                        ReadStorage<'a, Name>,
                        WriteStorage<'a, Position>,
                        WriteStorage<'a, InBackpack>
                      );


	fn run(&mut self, data: Self::SystemData) {
        let (player_entity, mut gamelog, entities, mut wants_drop, names, mut positions, mut backpack) = data;
		for (entity, to_drop) in (&entities, &wants_drop).join() {
            let mut dropper_pos : Position = Position{x:0, y:0};
            {
                let dropped_pos = positions.get(entity).unwrap();
                dropper_pos.x = dropped_pos.x;
                dropper_pos.y = dropped_pos.y;
            }
            positions.insert(to_drop.item, Position{ x : dropper_pos.x, y : dropper_pos.y }).expect("Unable to insert position");
            backpack.remove(to_drop.item);

            if entity == *player_entity {
                gamelog.entries.push(GameLog::drop_item_log(
					&names.get(to_drop.item).unwrap().name));
            }		
		}
		wants_drop.clear();
	}
}