use gfx::{self, *};

use ggez::mint::Point2;
use ggez::{conf, graphics, Context, ContextBuilder, GameResult};
use ggez::event::{self, EventHandler, KeyCode, KeyMods, MouseButton};
use specs::prelude::*;
use specs::saveload::{SimpleMarker, SimpleMarkerAllocator};
use bracket_lib::prelude::RandomNumberGenerator;

use std::env;
use std::path;
use std::collections::HashMap;

use serde::{Serialize, Deserialize};

mod spawner;
mod inventory_system;

mod gamelog;
pub use gamelog::*;

mod player;
pub use player::*;

mod monster_ai_system;
pub use monster_ai_system::*;

mod map;
pub use map::*;

mod component;
pub use component::*;

mod visibility_system;
pub use visibility_system::*;

mod map_indexing_system;
pub use map_indexing_system::*;

mod melee_combat_system;
pub use melee_combat_system::*;

mod damage_system;
pub use damage_system::*;

mod turnhealing_system;
pub use turnhealing_system::*;

mod saveload_system;

mod ui_helper;
mod imgui_helper;

mod random_table;
use bracket_lib::prelude::*;


const WINDOWSIZE_WIDTH: i32 = 40;
const WINDOWSIZE_HEIGHT: i32 = 19;
const PLAYER_WINDOW_WIDTH  : i32 = 29;
const PLAYER_WINDOW_HEIGHT : i32 = 19;

const TILESIZE: i32 = 32;

#[derive(PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub enum GameImage{
	Player,
	Dragon,
	Kobold,
	Potion,
	Scroll,
	Wall,
	Floor,
	DownStairs,
	Sword,
	Shield,
}

#[derive(PartialEq, Copy, Clone)]
pub enum MainMenuState {
	Waiting, NewGame, LoadGame, Quit
}

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
	AwaitingInput, PreRun, PlayerTurn, MonsterTurn, NextLevel, EndTurn,
	ShowInventory, ShowDropItem,
	ShowTargeting {range: i32, item: Entity},
	SaveGame, MainMenu {state: MainMenuState}
}

#[derive(PartialEq, Copy, Clone)]
pub enum MainMenuSelection { NewGame, LoadGame, Quit }


#[derive(PartialEq)]
pub enum RenderMode { Tile, Unicode }


fn map_to_world(x: i32,	y: i32) -> Point2<f32> {
	Point2 {
		x: (x * TILESIZE) as f32,
		y: (y * TILESIZE) as f32,
	}
}

fn main() -> GameResult {

	let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
		let mut path = path::PathBuf::from(manifest_dir);
			path.push("resources");
			path
		} else {
			path::PathBuf::from("./resources")
	};

	let cb = ContextBuilder::new("sushye", "esehara shigeo")
				.add_resource_path(resource_dir)
				.window_setup(conf::WindowSetup::default().title(&format!("{} {}", "sushye", "0.0.1")))
				.window_mode(conf::WindowMode::default().dimensions((WINDOWSIZE_WIDTH * TILESIZE) as f32, (WINDOWSIZE_HEIGHT * TILESIZE) as f32));


    let (ctx, event_loop) = &mut cb.build()?;
	let hidpi_factor = event_loop.get_primary_monitor().get_hidpi_factor() as f32;

    let game = &mut State::new(ctx, hidpi_factor)?;

	event::run(ctx, event_loop, game)
}

gfx_defines! {
	constant Dim {
		rate: f32 = "u_Rate",
	}
}

pub struct State {

	font: graphics::Font,

	mouse_x: f32,
	mouse_y: f32,

	images: HashMap<GameImage, graphics::Image>,
	imgui: imgui_helper::ImGuiWrapper,
	hidpi_factor: f32,

	render_mode: RenderMode,

	pub ecs: World
}

impl State {
	fn entities_to_remove_on_level_change(&mut self) -> Vec<Entity> {
		let entities = self.ecs.entities();
		let player = self.ecs.read_storage::<Player>();
		let backpack = self.ecs.read_storage::<InBackpack>();
		let player_entity = self.ecs.fetch::<Entity>();

		let mut to_delete : Vec<Entity> = Vec::new();
		for entity in entities.join() {
			let mut should_delete = true;

			let p = player.get(entity);
			if let Some(_p) = p { should_delete = false}
			let bp = backpack.get(entity);
			if let Some(bp) = bp {
				if bp.owner == *player_entity {should_delete = false}
			}
			if should_delete { to_delete.push(entity) }
		}
		to_delete
	}

	fn goto_next_level(&mut self) {
		let to_delete = self.entities_to_remove_on_level_change();
		for target in to_delete {
			self.ecs.delete_entity(target).unwrap();
		}
		let worldmap;
		let current_depth;
		{
			let mut worldmap_resource = self.ecs.write_resource::<Map>();
			current_depth = worldmap_resource.depth;
			worldmap = Map::new_map_rooms_and_corridors(current_depth + 1);
			*worldmap_resource = worldmap.clone();
		}

		for room in worldmap.rooms.iter().skip(1) {
			spawner::spawn_room(&mut self.ecs, room, current_depth + 1);
		}
	
		let (player_x, player_y) = worldmap.rooms[0].center();
		let mut player_position = self.ecs.write_resource::<Point>();
		*player_position = Point::new(player_x, player_y);
		let mut position_components = self.ecs.write_storage::<Position>();
		let player_entity = self.ecs.fetch::<Entity>();
		let player_pos_comp = position_components.get_mut(*player_entity);
		if let Some(player_pos_comp) = player_pos_comp {
			player_pos_comp.x = player_x;
			player_pos_comp.y = player_y;
		}

		let mut viewshed_components = self.ecs.write_storage::<Viewshed>();
		let vs = viewshed_components.get_mut(*player_entity);
		if let Some(vs) = vs { vs.dirty = true }
		
		let mut gamelog = self.ecs.fetch_mut::<gamelog::GameLog>();
		gamelog.entries.push(GameLog::goto_next_level_log());

	}


	fn run_systems(&mut self) {
		let mut mapindex = MapIndexingSystem{};
		mapindex.run_now(&self.ecs);
		let mut vis = VisibilitySystem {};
		vis.run_now(&self.ecs);
		let mut pickup = inventory_system::ItemCollectionSystem{};
		pickup.run_now(&self.ecs);
		let mut drop = inventory_system::ItemDropSystem {};
		drop.run_now(&self.ecs);
		let mut potions = inventory_system::ItemUseSystem{};
		potions.run_now(&self.ecs);
		let mut mob = MonsterAI{};
		mob.run_now(&self.ecs);
		let mut melee_combat = MeleeCombatSystem {};
		melee_combat.run_now(&self.ecs);
		let mut damage = DamageSystem {};
		damage.run_now(&self.ecs);
		damage_system::delete_the_dead(&mut self.ecs);

		let mut turnheal = TurnHealing {};
		turnheal.run_now(&self.ecs);

		self.ecs.maintain();
	}

	pub fn enum_to_unicode(self: &State, refer: &Renderable) -> graphics::TextFragment {
		match refer.image {
			GameImage::Player => graphics::TextFragment::new("＠")
				.color(graphics::Color::new(1.0, 1.0, 0.0, 1.0)),
			GameImage::Dragon => graphics::TextFragment::new("龍")
				.color(graphics::Color::new(1.0, 0.1, 0.1, 1.0)),
			GameImage::Kobold => graphics::TextFragment::new("妖")
				.color(graphics::Color::new(0.0, 1.0, 0.0, 1.0)),
			GameImage::Potion => graphics::TextFragment::new("壺")
				.color(graphics::Color::new(0.0, 0.5, 1.0, 1.0)),
			GameImage::Scroll => graphics::TextFragment::new("紙")
				.color(graphics::Color::new(1.0, 0.9, 0.7, 1.0)),
			GameImage::Sword => graphics::TextFragment::new("剣")
				.color(graphics::Color::new(0.5, 0.5, 0.0, 1.0)),
			GameImage::Shield => graphics::TextFragment::new("盾")
				.color(graphics::Color::new(0.0, 0.0, 0.8, 1.0)),
			_ => graphics::TextFragment::new("謎")
				.color(graphics::Color::new(1.0, 1.0, 1.0, 1.0))
		}
	}

    pub fn new(ctx: &mut Context, hidpi_factor: f32) -> GameResult<State> {
		let font = graphics::Font::new(ctx, "/unifont-13.ttf").unwrap();
		let mut prepare_images = HashMap::new();

		// loading Image
		prepare_images.insert(GameImage::Player, graphics::Image::new(ctx, "/player.png").unwrap());
		prepare_images.insert(GameImage::Dragon, graphics::Image::new(ctx, "/ice_dragon_new.png").unwrap());
		prepare_images.insert(GameImage::Kobold, graphics::Image::new(ctx, "/kobold.png").unwrap());
		prepare_images.insert(GameImage::Wall, graphics::Image::new(ctx, "/brick.png").unwrap());
		prepare_images.insert(GameImage::Floor, graphics::Image::new(ctx, "/floor.png").unwrap());
		prepare_images.insert(GameImage::DownStairs, graphics::Image::new(ctx, "/downstairs.png").unwrap());
		prepare_images.insert(GameImage::Potion, graphics::Image::new(ctx, "/potion.png").unwrap());
		prepare_images.insert(GameImage::Scroll, graphics::Image::new(ctx, "/scroll.png").unwrap());
		prepare_images.insert(GameImage::Sword, graphics::Image::new(ctx, "/sword.png").unwrap());
		prepare_images.insert(GameImage::Shield, graphics::Image::new(ctx, "/shield.png").unwrap());

		// Initialize => imgui
		let imgui = imgui_helper::ImGuiWrapper::new(ctx);

		// Create State
        let mut gs = State {		

			font: font,
			mouse_x: 0.0,
			mouse_y: 0.0,

			images: prepare_images,
			imgui: imgui,
			hidpi_factor: hidpi_factor,

			render_mode: RenderMode::Tile,

			ecs: World::new()
		};

		//
		// gs.ecs.register
		//

		gs.ecs.register::<SimpleMarker<SerializeMe>>();
		gs.ecs.register::<SerializationHelper>();
		gs.ecs.register::<Position>();
		gs.ecs.register::<Renderable>();
		
		gs.ecs.register::<Player>();
		gs.ecs.register::<Viewshed>();
		gs.ecs.register::<Monster>();
		gs.ecs.register::<SeenPlayer>();

		gs.ecs.register::<Name>();
		gs.ecs.register::<BlocksTile>();
		gs.ecs.register::<CombatStats>();
		gs.ecs.register::<WantsToMelee>();
		gs.ecs.register::<SufferDamage>();
		gs.ecs.register::<DurationTurnHeal>();

		gs.ecs.register::<InBackpack>();
		gs.ecs.register::<Item>();
		gs.ecs.register::<Potion>();

		gs.ecs.register::<Consumable>();
		gs.ecs.register::<ProvidesHealing>();
		gs.ecs.register::<Ranged>();
		gs.ecs.register::<InflictsDamage>();
		gs.ecs.register::<AreaOfEffect>();
		gs.ecs.register::<Paralyze>();

		gs.ecs.register::<DropItem>();
		gs.ecs.register::<WantsToPickupItem>();
		gs.ecs.register::<WantsToUseItem>();

		gs.ecs.register::<Equippable>();



		gs.ecs.insert(SimpleMarkerAllocator::<SerializeMe>::new());

		let map : Map = Map::new_map_rooms_and_corridors(1);
		let (player_x, player_y) = map.rooms[0].center();

		gs.ecs.insert(RandomNumberGenerator::new());

		let player_entity = spawner::player(&mut gs.ecs, player_x, player_y);
	
		for room in map.rooms.iter().skip(1) {
			spawner::spawn_room(&mut gs.ecs, room, 1);
		}

		gs.ecs.insert(map);

		gs.ecs.insert(Point::new(player_x, player_y));
		gs.ecs.insert(player_entity);
		gs.ecs.insert(RunState::MainMenu {state: MainMenuState::Waiting } );
		gs.ecs.insert(gamelog::GameLog{
			entries: vec![
				vec![
					graphics::TextFragment::new("Sushyの世界")
						.color(graphics::Color::new(1.0, 1.0, 0.0, 1.0)),
					graphics::TextFragment::new("へようこそ(Welcome to Sushy World)")]], 
				font: graphics::Font::new(ctx, "/PixelMplus.ttf").unwrap()}
			);
		Ok(gs)
    }
	
	fn draw_map(self: &State, ctx: &mut Context) {

		let mut viewshed = self.ecs.write_storage::<Viewshed>();
		let mut players = self.ecs.write_storage::<Player>();
		let positions = self.ecs.read_storage::<Position>();
		let map = self.ecs.fetch::<Map>();

		for (_player, _viewshed, position) in (&mut players, &mut viewshed, &positions).join() {
			for y in 0..PLAYER_WINDOW_HEIGHT {
				for x in 0..PLAYER_WINDOW_WIDTH {

					let fix_x = x + position.to_left() + 1;
					let fix_y = y + position.to_top() + 1;
					if fix_x < 0 || fix_y < 0 { continue; }

					let idx = map.xy_idx(fix_x, fix_y);
					if idx >= map.tiles.len() { continue; }

					let tile = map.tiles[idx];
					if map.revealed_tiles[idx]  {

						if self.render_mode == RenderMode::Tile {
							graphics::draw(
								ctx, self.images.get(&tile.tiletype.to_game_image()).unwrap(),
								graphics::DrawParam::default().dest(map_to_world(x, y)))
								.expect("cannot draw wall");
						} else {
							let use_text;
							match tile.background {
								None => {},
								Some(color) => {
									let background_rect = graphics::Mesh::new_rectangle(
												ctx,
												graphics::DrawMode::fill(),
												graphics::Rect::new(ui_helper::map_to_p(x), ui_helper::map_to_p(y),TILESIZE as f32, TILESIZE as f32),
												color)
											.unwrap();
									graphics::draw(ctx, &background_rect, graphics::DrawParam::default())
										.expect("Cannot draw background");
								}
							}

							match tile.tiletype {
								TileType::Floor => {
									use_text = graphics::TextFragment::new("床").color(
										graphics::Color::new(0.1, 0.1, 0.1, 0.4));
								},
								TileType::Wall => {
									use_text = graphics::TextFragment::new("壁")
										.color(graphics::Color::new(0.2, 0.0, 0.0, 1.0));
								},
								TileType::DownStairs => {
									use_text = graphics::TextFragment::new("門").color(
										graphics::Color::new(0.5, 0.5, 0.5, 1.0));							
								}
							}
							
							ui_helper::draw_tile_text(ctx, use_text, x, y, self.font);						
						}

						if !map.visible_tiles[idx] {
							let mask_rect = graphics::Mesh::new_rectangle(
										ctx,
										graphics::DrawMode::fill(),
										graphics::Rect::new(ui_helper::map_to_p(x), ui_helper::map_to_p(y),TILESIZE as f32, TILESIZE as f32),
										graphics::Color::new(0.0, 0.0, 0.0, 0.9))
									.unwrap();
							graphics::draw(ctx, &mask_rect, graphics::DrawParam::default())
								.expect("Cannot draw background");
						} else {
							let distance = bracket_lib::prelude::DistanceAlg::Pythagoras.distance2d(
								Point {x: fix_x, y: fix_y}, Point {x: position.x, y: position.y});
							
							if distance <= 2.0 {
								let use_alpha;
								if distance < 1.5 {
									use_alpha = 0.002;
								} else {
									use_alpha = 0.401 - (distance * 0.2);
								}
								let mask_rect = graphics::Mesh::new_rectangle(
											ctx,
											graphics::DrawMode::fill(),
											graphics::Rect::new(ui_helper::map_to_p(x), ui_helper::map_to_p(y),TILESIZE as f32, TILESIZE as f32),
											graphics::Color::new(1.0, 1.0, 0.0, use_alpha))
										.unwrap();
								graphics::draw(ctx, &mask_rect, graphics::DrawParam::default())								
									.expect("Cannot draw background");
							} else {
								let mask_rect = graphics::Mesh::new_rectangle(
											ctx,
											graphics::DrawMode::fill(),
											graphics::Rect::new(ui_helper::map_to_p(x), ui_helper::map_to_p(y),TILESIZE as f32, TILESIZE as f32),
											graphics::Color::new(0.0, 0.0, 0.0, distance * 0.10))
										.unwrap();
								graphics::draw(ctx, &mask_rect, graphics::DrawParam::default())	
									.expect("Cannot draw background");
							
							}						
						}

						graphics::clear_shader(ctx);
					}
				}	
			}
		}
	}

	fn draw_title(&mut self, ctx: &mut Context) {
		self.imgui.render(ctx, &self.ecs, self.hidpi_factor);
	}

	fn draw_maingame(&mut self, ctx: &mut Context, runstatus: RunState) {
		let map = self.ecs.fetch::<Map>();
		self.draw_map(ctx);

		if let RunState::ShowTargeting {range, item} = runstatus {
			ui_helper::draw_ranged_target(&self.ecs, ctx, range);
			let aoe = self.ecs.read_storage::<AreaOfEffect>();
			let is_aoe_item = aoe.get(item);
			match is_aoe_item {
				None => {},
				Some(aoe_item) => ui_helper::draw_aoe_radius(
					ctx, &self.ecs, self.mouse_x, self.mouse_y, aoe_item.radius) 
			}
		}
				
		let players = self.ecs.read_storage::<Player>();
		let renderables = self.ecs.read_storage::<Renderable>();
		let positions = self.ecs.read_storage::<Position>();

		let mut data = (&positions, &renderables).join().collect::<Vec<_>>();
		data.sort_by(|&a, &b| a.1.render_layer.cmp(&b.1.render_layer));
		for (_player, player_pos) in (&players, &positions).join() {
			for (pos, render) in data.iter() {
				if pos.x < player_pos.to_left() || pos.y < player_pos.to_top() ||
					pos.x > player_pos.to_right() || pos.y > player_pos.to_buttom() { continue; }

				let idx = map.xy_idx(pos.x, pos.y);
				let draw_position = Position{
					x: pos.x - (player_pos.to_left() + 1),
					y: pos.y - (player_pos.to_top() + 1)}.map_to_world();

				if map.visible_tiles[idx] {
					if ui_helper::p_to_map(self.mouse_x) + player_pos.to_left() >= 0 && ui_helper::p_to_map(self.mouse_y) + player_pos.to_top() >= 0 {
						if idx == map.xy_idx_from_mouse(self.mouse_x, self.mouse_y, *player_pos) {
							ui_helper::draw_object_focus_rect(ctx, &self.ecs, self.mouse_x, self.mouse_y, *pos);
							ui_helper::draw_tooltip_with_mouse_motion(ctx, &self.ecs, self.mouse_x, self.mouse_y, self.font);		
						}
					}
					if self.render_mode == RenderMode::Tile {
						graphics::draw(
							ctx,
							self.images.get(&render.image).unwrap(),
							graphics::DrawParam::new().dest(draw_position)
						).unwrap();
					} else {
						ui_helper::draw_tile_text(
							ctx,
							self.enum_to_unicode(&render),
							pos.x - (player_pos.to_left() + 1), 
							pos.y - (player_pos.to_top() + 1),
							self.font);
					
					}
				}
			}
		}

		ui_helper::draw_mouse_pos(ctx, self.mouse_x, self.mouse_y);
		// UI
		ui_helper::draw_message_window(ctx, &self.ecs, self.font);
		self.imgui.states_window(ctx, &self.ecs, self.hidpi_factor);
		match runstatus {
			RunState::ShowInventory =>
				ui_helper::draw_inventory_window(
					&self, ctx, &self.ecs, self.font, Point2 {x: self.mouse_x, y: self.mouse_y}, &self.render_mode),
			RunState::ShowDropItem =>
				ui_helper::draw_inventory_window(
					&self, ctx, &self.ecs, self.font, Point2 {x: self.mouse_x, y: self.mouse_y}, &self.render_mode),
			_ => {}
		}
	}
}



impl EventHandler for State {

	fn quit_event(&mut self, ctx: &mut Context) -> bool{
		saveload_system::save_game(&mut self.ecs);
		false
	}

	fn key_down_event (&mut self, _ctx: &mut Context, keycode: KeyCode, keymod: KeyMods, _repeat: bool) {
	
		let mut newrunstate;
		{
			let runstate = self.ecs.fetch::<RunState>();
			newrunstate = *runstate;
		}
		match newrunstate{		
			RunState::ShowInventory => {
				match keymod {
					KeyMods::NONE => {
						match keycode {
							KeyCode::Escape => newrunstate = RunState::AwaitingInput,
							_ =>{ 
								// keycode  "a" -> 10
								newrunstate = try_use_item(&mut self.ecs, (keycode as i32) - 10);
							}
						}
					},
					_ => {}
				}		
			}
			RunState::ShowDropItem => {
				match keymod {
					KeyMods::NONE => {
						match keycode {
							KeyCode::Escape => newrunstate = RunState::AwaitingInput,
							_ =>{ 
								// keycode  "a" -> 10
								newrunstate = try_drop_item(&mut self.ecs, (keycode as i32) - 10);
							}
						}
					},
					_ => {}
				}				
			} 
			RunState::ShowTargeting {range:_ , item:_ } => {
				match keymod {
					KeyMods::NONE => {
						match keycode {
							KeyCode::Escape => newrunstate = RunState::AwaitingInput,
							_ => {}
						}
					}
					_ => {}
				}
			}
			RunState::AwaitingInput => {
				match keymod {
					KeyMods::SHIFT => {
						match keycode {
							KeyCode::Left => newrunstate = try_move_player(-1, -1, &mut self.ecs),
							KeyCode::Right => newrunstate = try_move_player(1, -1, &mut self.ecs),
							KeyCode::Period => {newrunstate = try_next_level(&mut self.ecs)}
							_ => { return; }
						}
					},
					KeyMods::CTRL  => {
						match keycode {
							KeyCode::Left =>  newrunstate = try_move_player(-1, 1, &mut self.ecs),
							KeyCode::Right => newrunstate = try_move_player(1, 1, &mut self.ecs),
							_ => { return; }
						}
					},
					KeyMods::NONE => {
						match keycode {
							KeyCode::Left  => newrunstate = try_move_player(-1, 0, &mut self.ecs),
							KeyCode::Right => newrunstate = try_move_player(1, 0, &mut self.ecs),
							KeyCode::Up    => newrunstate = try_move_player(0, -1, &mut self.ecs),
							KeyCode::Down  => newrunstate = try_move_player(0, 1, &mut self.ecs),
							KeyCode::D     => newrunstate = RunState::ShowDropItem,
							KeyCode::G     => newrunstate = get_item(&mut self.ecs),
							KeyCode::I     => newrunstate = RunState::ShowInventory,
							KeyCode::S     => newrunstate = RunState::PlayerTurn,
							KeyCode::F11   => {
								if self.render_mode == RenderMode::Tile {
									self.render_mode = RenderMode::Unicode;
								} else {
									self.render_mode = RenderMode::Tile;
								}
							}
						_  => {}
						}
					},
					_ => { return; }
				}		
			}
			_ => {},
		}

		{
			let mut runwriter = self.ecs.write_resource::<RunState>();
			*runwriter = newrunstate;
		}
	}


    fn mouse_motion_event(&mut self, _ctx: &mut Context, x: f32, y: f32, _xrel: f32, _yrel: f32) {
		self.mouse_x = x;
		self.mouse_y = y;
		self.imgui.update_mouse_pos(x, y);
	}

	fn mouse_button_up_event(&mut self, _ctx: &mut Context, button: MouseButton, x: f32, y:f32) {
		self.imgui.update_mouse_down((false, false, false));
	}

    fn mouse_button_down_event(&mut self, _ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
		let mut newrunstate;
		{
			let runstate = self.ecs.fetch::<RunState>();
			newrunstate = *runstate;
		}
		let mut get_another_event = false;

		match newrunstate {
			RunState::ShowTargeting {range, item} => {
				newrunstate = ui_helper::try_target_object(&self.ecs, x, y, item, range);
				get_another_event = true;
			},
			_ => {}
		}

		if !get_another_event {
			self.imgui.update_mouse_down(
			(button == MouseButton::Left,
			 button == MouseButton::Right,
			 button == MouseButton::Middle));
		}

		{
			let mut runwriter = self.ecs.write_resource::<RunState>();
			*runwriter = newrunstate;
		}

	}


    fn update(&mut self, ctx: &mut Context) -> GameResult {
		let mut newrunstate;
		{
			let runstate = self.ecs.fetch::<RunState>();
			newrunstate = *runstate;
		}

		match newrunstate {
			RunState::MainMenu { state } => {
				match state {
					MainMenuState::NewGame => {
						//TODO: initialize game function
						newrunstate = RunState::PreRun;
					},
					MainMenuState::Quit => {
						::std::process::exit(0);
					},
					MainMenuState::LoadGame => {
						saveload_system::load_game(&mut self.ecs);
						newrunstate = RunState::AwaitingInput;
						saveload_system::delete_save();
					},
					MainMenuState::Waiting => {}
				}
			}
			RunState::PreRun => {
				self.run_systems();
				newrunstate = RunState::AwaitingInput;
			}
			RunState::AwaitingInput => {}
			RunState::NextLevel => {
				self.goto_next_level();
				newrunstate = RunState::PreRun;
			}
			RunState::PlayerTurn => {
				self.run_systems();
				newrunstate = RunState::SaveGame;
			}
			RunState::SaveGame => {
				// It's currently impossible to save every turn.
				newrunstate = RunState::MonsterTurn;
			}
			RunState::MonsterTurn => {
				self.run_systems();
				self.ecs.maintain();
				newrunstate = RunState::EndTurn;
			}
			RunState::EndTurn => {
				damage_system::delete_the_dead(&mut self.ecs);	
				newrunstate = RunState::AwaitingInput;
			}
			RunState::ShowInventory => {}
			RunState::ShowDropItem => {}
			RunState::ShowTargeting{range, item} => {
			}
		}

		{
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = newrunstate;
        }
		 Ok(())
	}

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, graphics::BLACK);
		let runstatus = *(self.ecs.fetch::<RunState>()).clone();
		match runstatus {
			RunState::MainMenu { state } => {
				self.draw_title(ctx);
			}
			_ => {
				self.draw_maingame(ctx, runstatus);
			}
		}

        graphics::present(ctx)
    }

}