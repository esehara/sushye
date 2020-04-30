use specs::prelude::*;
use specs_derive::*;
use serde::{Serialize, Deserialize};
use specs::saveload::{Marker, ConvertSaveload};
use specs::error::NoError;
use ggez::graphics::Color;
use super:: {GameImage, PLAYER_WINDOW_HEIGHT, PLAYER_WINDOW_WIDTH,
	Point2, Point, TILESIZE, Map};

pub struct SerializeMe;


#[derive(Component, Clone, Serialize, Deserialize)]
pub struct Renderable {
	pub image: GameImage,
	pub render_layer: i32,

	#[serde(skip_serializing)]
	#[serde(skip_deserializing)]
	pub background: Option<Color>,
}

#[derive(Component, Clone, Copy, ConvertSaveload)]
pub struct Position{
	pub x: i32,
	pub y: i32,
}

impl Position {
	pub fn to_top(&self) -> i32 {
		( -1 * (PLAYER_WINDOW_HEIGHT / 2)) + self.y
	}

	pub fn to_left(&self) -> i32 {
		(-1 * (PLAYER_WINDOW_WIDTH / 2)) + self.x
	}

	pub fn to_right(&self) -> i32 {
		PLAYER_WINDOW_WIDTH / 2 + 1 + self.x
	}

	pub fn to_buttom(&self) -> i32 {
		PLAYER_WINDOW_HEIGHT / 2 + 1 + self.y
	}

	pub fn map_to_world(self: &Position) -> Point2<f32> {
		let x = self.x;
		let y = self.y;
		Point2 { x: x as f32 * TILESIZE as f32, 
				 y: y as f32 * TILESIZE as f32}
	}
}


pub fn point_to_left(p: Point) -> i32 {
	(-1 * (PLAYER_WINDOW_WIDTH / 2)) + p.x
}

pub fn point_to_top(p: Point) -> i32 {
	(-1 * (PLAYER_WINDOW_HEIGHT / 2)) + p.y
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Viewshed {
	pub visible_tiles: Vec<Point>,
	pub range: i32,
	pub dirty: bool,
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Monster {}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct SeenPlayer {
	pub point: Option<Point>,
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct Name {
	pub name: String
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct BlocksTile {}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct CombatStats {
	pub max_hp: i32,
	pub hp: i32,
	pub defense: i32,
	pub power: i32
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct DurationTurnHeal {
	pub time: i32,
}

#[derive(Component, Debug, Clone, ConvertSaveload)]
pub struct WantsToMelee {
	pub target : Entity
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Item {}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Consumable {}


#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct Potion {
	pub heal_amount: i32
}

#[derive(Component, Debug, Clone, ConvertSaveload)]
pub struct InBackpack {
	pub owner : Entity
}

#[derive(Component, Debug, Clone, ConvertSaveload)]
pub struct WantsToPickupItem {
	pub collected_by: Entity,
	pub item: Entity
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct SufferDamage {
	pub amount: Vec<i32>
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct WantsToUseItem {
    pub item : Entity,
	pub target: Option<Point>
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct DropItem {
	pub item: Entity
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct ProvidesHealing {
    pub heal_amount : i32
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct Ranged {
	pub range: i32
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct InflictsDamage{
	pub damage: i32
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct AreaOfEffect {
	pub radius: i32
}

#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct Paralyze {
	pub turns: i32
}

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum EquipmentSlot {Melee, Shield}

#[derive(Component, Clone, ConvertSaveload)]
pub struct Equippable {
	pub slot : EquipmentSlot
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Equipped {
	pub owner: Entity,
	pub slot: EquipmentSlot
}


impl SufferDamage {
	pub fn new_damage(store: &mut WriteStorage<SufferDamage>, victim: Entity, amount: i32) {
		if let Some(suffering) = store.get_mut(victim) {
			suffering.amount.push(amount);
		} else {
			let dmg = SufferDamage {amount : vec![amount] };
			store.insert(victim, dmg).expect("Unable to insert Damage");
		}
	}
}

#[derive(Component, Serialize, Deserialize, Clone)]
pub struct SerializationHelper {
	pub map: super::map::Map
}