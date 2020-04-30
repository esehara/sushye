use std::cmp::{max, min};
use ggez::graphics::Color;
use bracket_lib::prelude::*;
use specs::Entity;
use super:: {TILESIZE, Position, ui_helper, GameImage};
use serde::{Serialize, Deserialize};

pub const MAPSIZE_WIDTH: usize = 64;
pub const MAPSIZE_HEIGHT: usize = 64;
pub const MAPSIZE_COUNT: usize = MAPSIZE_HEIGHT * MAPSIZE_WIDTH;
pub	const MIN_ROOMS : i32 = 10;
pub const TRY_ROOMS : i32 = 3;

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum TileType {
	Wall, Floor, DownStairs
}

impl TileType {
	pub fn to_game_image(self) -> GameImage {
		match self {
			TileType::Wall => GameImage::Wall,
			TileType::Floor => GameImage::Floor,
			TileType::DownStairs => GameImage::DownStairs,
		}
	}
}

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub struct Tile {
	pub tiletype: TileType,

	#[serde(skip_serializing)]
	#[serde(skip_deserializing)]
	pub background: Option<Color>,
}

impl Tile {
	pub fn set_background(&mut self,rng: &mut bracket_lib::prelude::RandomNumberGenerator) {
		let r = rng.range(0, 3);
		match self.tiletype {
			TileType::Wall => { 
				match r {
					0 => self.background = Some(Color::new(0.7, 0.2, 0.2, 1.0)),
					1 => self.background = Some(Color::new(0.8, 0.2, 0.2, 1.0)),
					2 => self.background = Some(Color::new(0.9, 0.2, 0.2, 1.0)),
					_ => {},
				}
			}
			TileType::Floor => { 
				match r {
					0 => self.background = Some(Color::new(0.1, 0.1, 0.1, 1.0)),
					1 => self.background = Some(Color::new(0.15, 0.15, 0.15, 1.0)),
					2 => self.background = Some(Color::new(0.2, 0.2, 0.2, 1.0)),
					_ => {},
				}
			}
			TileType::DownStairs => {}
		}
	}		
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Map {
	pub tiles: Vec<Tile>,
	pub revealed_tiles: Vec<bool>,
	pub visible_tiles: Vec<bool>,
	pub blocked: Vec<bool>,
	pub rooms: Vec<Rect>,

	pub width: i32,
	pub height: i32,
	pub depth: i32,

	#[serde(skip_serializing)]
	#[serde(skip_deserializing)]
	pub tile_content: Vec<Vec<Entity>>,
}


impl Algorithm2D for Map {
	fn dimensions(&self) -> Point {
		Point::new(self.width, self.height)
	}
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Rect {
    pub x1 : i32,
    pub x2 : i32,
    pub y1 : i32,
    pub y2 : i32
}

impl Rect {
    pub fn new(x:i32, y: i32, w:i32, h:i32) -> Rect {
        Rect{x1:x, y1:y, x2:x+w, y2:y+h}
    }

    // Returns true if this overlaps with other
    pub fn intersect(&self, other:&Rect) -> bool {
        self.x1 <= other.x2 && self.x2 >= other.x1 && self.y1 <= other.y2 && self.y2 >= other.y1
    }

    pub fn center(&self) -> (i32, i32) {
        ((self.x1 + self.x2)/2, (self.y1 + self.y2)/2)
    }
}

impl Map {

	pub fn xy_idx(&self, x: i32, y: i32) -> usize {
		(y as usize * self.width as usize) + x as usize
	}
	pub fn xy_idx_from_mouse(&self, x: f32, y: f32, player_pos: Position) -> usize {
		self.xy_idx(ui_helper::p_to_map(x) + player_pos.to_left() + 1 , ui_helper::p_to_map(y) + player_pos.to_top() + 1)
	}

	fn is_exit_valid(&self, x:i32, y:i32) -> bool {
		if x < 1 || x > self.width-1|| y < 1 || y > self.height-1 { return false; }
		let idx = self.xy_idx(x, y);
		!self.blocked[idx as usize]
	}

	fn apply_horizontal_tunnel(&mut self, x1:i32, x2:i32, y:i32) {
		for x in min(x1,x2) ..= max(x1,x2) {
			let idx = self.xy_idx(x, y);
			if idx > 0 && idx < (MAPSIZE_HEIGHT * MAPSIZE_WIDTH) as usize {
				self.tiles[idx as usize].tiletype = TileType::Floor;
			}
		}
	}


	fn apply_vertical_tunnel(&mut self, y1:i32, y2:i32, x:i32) {
		for y in min(y1,y2) ..= max(y1,y2) {
			let idx = self.xy_idx(x, y);
			if idx > 0 && idx < (MAPSIZE_HEIGHT * MAPSIZE_WIDTH) as usize {
				let idx = self.xy_idx(x, y);
				self.tiles[idx as usize].tiletype = TileType::Floor;
			}
		}
	}

	fn apply_room_to_map(&mut self, room : &Rect) {
		for y in room.y1 +1 ..= room.y2 {
			for x in room.x1 + 1 ..= room.x2 {
				let idx = self.xy_idx(x, y);
				self.tiles[idx as usize].tiletype = TileType::Floor;
			}
		}
	}
	

	fn new_room_and_corridor(map: &mut Map) {

		const MIN_SIZE  : i32 = 5;
		const MAX_SIZE  : i32 = 15;
	
		let mut rng = RandomNumberGenerator::new();
		let w = rng.range(MIN_SIZE, MAX_SIZE);
		let h = rng.range(MIN_SIZE, MAX_SIZE);
		let x = rng.roll_dice(1, (MAPSIZE_WIDTH as i32) - w - 1) - 1;
		let y = rng.roll_dice(1, (MAPSIZE_HEIGHT as i32) - h - 1) - 1;

		let new_room = Rect::new(x, y, w, h);
		let mut ok = true;
		for other_room in map.rooms.iter(){
			if new_room.intersect(other_room) { ok = false }
		}

		if ok {
			map.apply_room_to_map(&new_room);
			if !map.rooms.is_empty() {
				let (new_x, new_y) = new_room.center();
				let (prev_x, prev_y) = map.rooms[map.rooms.len() - 1].center();
				
                if rng.range(0,2) == 1 {
                    map.apply_horizontal_tunnel(prev_x, new_x, prev_y);
                    map.apply_vertical_tunnel(prev_y, new_y, new_x);
                } else {
                    map.apply_vertical_tunnel(prev_y, new_y, prev_x);
                    map.apply_horizontal_tunnel(prev_x, new_x, new_y);
                }
			}
			map.rooms.push(new_room);
		}

	}

	pub fn populate_blocked(&mut self) {
		for (i, tile) in self.tiles.iter_mut().enumerate() {
			self.blocked[i] = tile.tiletype == TileType::Wall;
		}
	}

	pub fn clear_content_index(&mut self){
		for content in self.tile_content.iter_mut() {
			content.clear();
		}
	}

	fn set_background(&mut self) {
		let mut rng = bracket_lib::prelude::RandomNumberGenerator::new();
		for tile in self.tiles.iter_mut() {
			tile.set_background(&mut rng);
		}
	}


	pub fn new_map_rooms_and_corridors(depth: i32) -> Map {
		const MAPSIZE_FOR_INIT : usize = (MAPSIZE_WIDTH * MAPSIZE_HEIGHT) as usize;
		let mut map = Map {
			visible_tiles: vec![false; (MAPSIZE_WIDTH * MAPSIZE_HEIGHT) as usize],
			revealed_tiles: vec![false; (MAPSIZE_WIDTH * MAPSIZE_HEIGHT) as usize],
			blocked: vec![false; (MAPSIZE_WIDTH * MAPSIZE_HEIGHT) as usize ],

			tiles: vec![Tile {tiletype: TileType::Wall, background: None} ; (MAPSIZE_WIDTH * MAPSIZE_HEIGHT) as usize],
			rooms: Vec::new(),

			width: MAPSIZE_WIDTH as i32,
			height: MAPSIZE_HEIGHT as i32,

			depth: depth,

			tile_content: vec![Vec::new(); MAPSIZE_FOR_INIT],
		};

		while map.rooms.len() < MIN_ROOMS as usize {
			for _ in 0..TRY_ROOMS {
				Map::new_room_and_corridor(&mut map);
			}
		}
		
		map.set_background();
		
		let stairs_position = map.rooms[map.rooms.len() - 1].center();
		let stairs_idx = map.xy_idx(stairs_position.0, stairs_position.1);
		map.tiles[stairs_idx].tiletype = TileType::DownStairs;
		map
	}
}


impl BaseMap for Map {
	fn is_opaque(&self, idx: usize) -> bool {
		self.tiles[idx as usize].tiletype == TileType::Wall
	}

	fn get_pathing_distance(&self, idx1:usize, idx2:usize) -> f32 {
        let w = self.width as usize;
        let p1 = Point::new(idx1 % w, idx1 / w);
        let p2 = Point::new(idx2 % w, idx2 / w);
        DistanceAlg::Pythagoras.distance2d(p1, p2)
    }

	fn get_available_exits(&self, idx: usize) ->  SmallVec<[(usize, f32); 10]> {
		let mut exits : SmallVec<[(usize, f32); 10]>  = SmallVec::new();
		let x = idx as i32 % self.width;
		let y = idx as i32 / self.width;
		let w = self.width as usize;

		if self.is_exit_valid(x - 1, y) { exits.push((idx - 1, 1.0))};
		if self.is_exit_valid(x + 1, y) { exits.push((idx + 1, 1.0))};
		if self.is_exit_valid(x, y - 1) { exits.push((idx - w, 1.0))};
		if self.is_exit_valid(x, y + 1) { exits.push((idx + w, 1.0))};

		if self.is_exit_valid(x-1, y-1) { exits.push(((idx-w)-1, 1.45)); }
		if self.is_exit_valid(x+1, y-1) { exits.push(((idx-w)+1, 1.45)); }
		if self.is_exit_valid(x-1, y+1) { exits.push(((idx+w)-1, 1.45)); }
		if self.is_exit_valid(x+1, y+1) { exits.push(((idx+w)+1, 1.45)); }


		exits
	}

}
