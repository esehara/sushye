use specs::prelude::*;
use specs_derive::*;
use ggez::graphics::Color;
use super:: {GameImage, PLAYER_WINDOW_HEIGHT, PLAYER_WINDOW_WIDTH,
	Point2, Point, TILESIZE};

#[derive(Component)]
pub struct Renderable {
	pub image: GameImage,
	pub background: Option<Color>,
	pub render_layer: i32
}

#[derive(Component, Clone, Copy)]
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

#[derive(Component)]
pub struct Viewshed {
	pub visible_tiles: Vec<Point>,
	pub range: i32,
	pub dirty: bool,
}

#[derive(Component)]
pub struct Monster {}

#[derive(Component)]
pub struct SeenPlayer {
	pub point: Option<Point>,
}

#[derive(Component, Debug)]
pub struct Name {
	pub name: String
}

#[derive(Component, Debug)]
pub struct BlocksTile {}

#[derive(Component, Debug)]
pub struct CombatStats {
	pub max_hp: i32,
	pub hp: i32,
	pub defense: i32,
	pub power: i32
}
#[derive(Component, Debug)]
pub struct DurationTurnHeal {
	pub time: i32,
}

#[derive(Component, Debug, Clone)]
pub struct WantsToMelee {
	pub target : Entity
}

#[derive(Component, Debug)]
pub struct Item {}

#[derive(Component, Debug)]
pub struct Consumable {}


#[derive(Component, Debug)]
pub struct Potion {
	pub heal_amount: i32
}

#[derive(Component, Debug, Clone)]
pub struct InBackpack {
	pub owner : Entity
}

#[derive(Component, Debug, Clone)]
pub struct WantsToPickupItem {
	pub collected_by: Entity,
	pub item: Entity
}

#[derive(Component, Debug)]
pub struct SufferDamage {
	pub amount: Vec<i32>
}

#[derive(Component, Debug)]
pub struct WantsToUseItem {
    pub item : Entity,
	pub target: Option<Point>
}

#[derive(Component, Debug)]
pub struct DropItem {
	pub item: Entity
}

#[derive(Component, Debug)]
pub struct ProvidesHealing {
    pub heal_amount : i32
}

#[derive(Component, Debug)]
pub struct Ranged {
	pub range: i32
}

#[derive(Component, Debug)]
pub struct InflicsDamage{
	pub damage: i32
}

#[derive(Component, Debug)]
pub struct AreaOfEffect {
	pub radius: i32
}

#[derive(Component, Debug)]
pub struct Paralyze {
	pub turns: i32
}

#[derive(PartialEq, Copy, Clone)]
pub enum EquipmentSlot {Melee, Shild}

#[derive(Component, Clone)]
pub struct  Equippable {
	pub slot : EquipmentSlot
}

#[derive(Component)]
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