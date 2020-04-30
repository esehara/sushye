use bracket_lib::prelude:: {RandomNumberGenerator };
use specs::prelude::*;
use std::collections::HashMap;

use super::{CombatStats, Player, Renderable, Name, MAPSIZE_WIDTH,
			Consumable, ProvidesHealing, Position, Viewshed, Monster, BlocksTile, SeenPlayer,
			GameImage, map, Item, Potion, Ranged, InflicsDamage, AreaOfEffect,
			Paralyze, DurationTurnHeal, Equippable, EquipmentSlot,
			random_table::RandomTable, map::Rect};

const MAX_MONSTERS : i32 = 4;
const MAX_ITEMS : i32 = 2;
pub fn player(ecs : &mut World, player_x : i32, player_y : i32) -> Entity {
	ecs
			.create_entity()
			.with(Position {x: player_x, y: player_y})
			.with(Renderable {image: GameImage::Player, render_layer: 2, background: None})
			.with(Player {})
			.with(Viewshed {visible_tiles: Vec::new(), range: 8, dirty: true})
			.with(CombatStats { max_hp: 30, hp: 30, defense: 2, power: 5})
			.with(Name {name: "Player".to_string() })
			.with(DurationTurnHeal {time: 0})
			.build()
}

fn room_table(map_depth: i32) -> RandomTable {
	RandomTable::new()
		.add("Kobold", 10)
		.add("Dragon", 1 + map_depth)
		.add("HealPotion", 5)
		.add("FireballScroll", 2 + map_depth)
		.add("ParalyzeScroll", 2 + map_depth)
		.add("MagicMissileScroll", 4)
		.add("IronSword", 3)
		.add("IronShild", 3)
}

fn dragon(ecs: &mut World, x: i32, y: i32) { 
	monster(ecs, x, y, GameImage::Dragon, "Dragoso", 24, 1, 5); 
}

fn kobold(ecs: &mut World, x: i32, y: i32) { 
	monster(ecs, x, y, GameImage::Kobold, "Kobolso", 16, 1, 4);
}

fn monster<S : ToString>(ecs: &mut World, x: i32, y: i32, image : GameImage, name : S, hp: i32, defence: i32, power: i32) {
    ecs.create_entity()
        .with(Position{ x, y })
        .with(Renderable{ image: image, render_layer: 2, background: None})
        .with(Viewshed{ visible_tiles : Vec::new(), range: 8, dirty: true })
        .with(Monster{})
        .with(Name{ name : name.to_string() })
        .with(BlocksTile{})
        .with(CombatStats{ max_hp: 16, hp: hp, defense: defence, power: power})
        .with(DurationTurnHeal {time: 0})
		.with(SeenPlayer{point: None})
		.build();
}

fn iron_sword(ecs: &mut World, x:i32, y:i32) {
	ecs.create_entity()
		.with(Position{x: x, y: y})
		.with(Renderable { image: GameImage::Sword, render_layer: 1, background: None})
		.with(Name {name: "Iron Sword".to_string()})
		.with(Item{})
		.with(Equippable { slot: EquipmentSlot::Melee })
		.build();
}

fn iron_shild(ecs: &mut World, x:i32, y:i32) {
	ecs.create_entity()
		.with(Position{x: x, y: y})
		.with(Renderable { image: GameImage::Shild, render_layer: 1, background: None})
		.with(Name {name: "Iron Shild".to_string()})
		.with(Item{})
		.with(Equippable {slot: EquipmentSlot::Shild })
		.build();
}

fn health_potion(ecs: &mut World, x: i32, y: i32) {
	ecs.create_entity()
		.with(Position{x: x, y: y})
		.with(Renderable { image: GameImage::Potion, render_layer: 1, background: None})
		.with(Name {name: "Heal Potion".to_string()})
		.with(Item{})
		.with(Consumable {})
		.with(ProvidesHealing {heal_amount: 8})
		.with(Potion { heal_amount: 8})
		.build();
}

fn magic_missile_scroll(ecs: &mut World, x: i32, y: i32) {
	ecs.create_entity()
		.with(Position{x: x, y: y})
		.with(Renderable { image: GameImage::Scroll, render_layer: 1, background: None})
		.with(Name {name: "Magic Missle Scroll".to_string()})
		.with(Item{})
		.with(Consumable {})
		.with(Ranged {range: 6})
		.with(InflicsDamage { damage: 8})
		.build();
}

fn fireball_scroll(ecs: &mut World, x: i32, y: i32) {
	ecs.create_entity()
		.with(Position{x: x, y: y})
		.with(Renderable { image: GameImage::Scroll, render_layer: 1, background: None})
		.with(Name {name: "Fireball Scroll".to_string()})
		.with(Item{})
		.with(Consumable {})
		.with(Ranged {range: 6})
		.with(InflicsDamage { damage: 8})
		.with(AreaOfEffect{ radius: 3})
		.build();
}

fn paralyze_scroll(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position{ x, y })
        .with(Renderable{ image: GameImage::Scroll, render_layer: 1, background: None})
        .with(Name{ name : "Paralyze Scroll (麻痺の巻物) ".to_string() })
        .with(Item{})
        .with(Paralyze { turns: 3})
        .with(Ranged{ range: 6 })
        .build();
}

fn room_inside_idx(room: &map::Rect, ecs: &mut World) -> usize {
	let mut rng = ecs.write_resource::<RandomNumberGenerator>();
	let x = (room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1))) as usize;
	let y = (room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1))) as usize;
    (y * map::MAPSIZE_WIDTH) + x
}

pub fn spawn_room(ecs: &mut World, room : &Rect, map_depth: i32) {
	let spawn_table = room_table(map_depth);
	let mut spawn_points : HashMap<usize, String> = HashMap::new();
	{
		let mut rng = ecs.write_resource::<RandomNumberGenerator>();
		let num_spawns = rng.roll_dice(1, MAX_MONSTERS + 3) + (map_depth - 1) - 3;
		for _i in 0..num_spawns {
			let mut added = false;
			let mut tries = 0;
			while !added && tries < 20 {
				let x = (room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1))) as usize;
				let y = (room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1))) as usize;
				let idx = (y * MAPSIZE_WIDTH) + x;
				if !spawn_points.contains_key(&idx) {
					spawn_points.insert(idx as usize, spawn_table.roll(&mut rng));
					added = true;
				} else {
					tries += 1;
				}
			}
		}
	}

	for spawn in spawn_points.iter() {
		let x = (*spawn.0 % MAPSIZE_WIDTH) as i32;
		let y = (*spawn.0 / MAPSIZE_WIDTH) as i32;
		match spawn.1.as_ref() {
			"Kobold" => kobold(ecs, x, y),
			"Dragon" => dragon(ecs, x, y),
			"HealPotion" => health_potion(ecs, x, y),
			"FireBallScroll" => fireball_scroll(ecs, x, y),
			"MagicMissileScroll" => magic_missile_scroll(ecs, x, y),
			"ParalyzeScroll" => paralyze_scroll(ecs, x, y),
			"IronSword" => iron_sword(ecs, x, y),
			"IronShild" => iron_shild(ecs, x, y),
			_ => {}
		}
	}
}
