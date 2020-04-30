use ggez::graphics:: {TextFragment, Color};

pub struct GameLog {
    pub entries : Vec<Vec<TextFragment>>,
	pub font: ggez::graphics::Font
}

const NAME_COLOR : Color = Color::new(1.0, 1.0, 0.0, 1.0);

impl GameLog {

	pub fn dead_log(name: &String) -> Vec<TextFragment> {
		vec![
			TextFragment::new(name.to_string()).color(
				Color::new(1.0, 0.0, 0.0, 1.0)),
			TextFragment::new("は死んだ。").color(
				Color::new(1.0, 0.0, 0.0, 1.0))
		]
	}

	pub fn get_item_log(name: &String) -> Vec<TextFragment> {
		vec![
			TextFragment::new(name.to_string()).color(
				Color::new(1.0, 1.0, 0.0, 1.0)),
			TextFragment::new("を拾った。").color(
				Color::new(1.0, 1.0, 1.0, 1.0))
			]
	}

	pub fn drop_item_log(name: &String) -> Vec<TextFragment> {
		vec![
			TextFragment::new(name.to_string()).color(
				Color::new(1.0, 1.0, 0.0, 1.0)),
			TextFragment::new("を落とした。").color(
				Color::new(1.0, 1.0, 1.0, 1.0))
			]	
	}
	
	pub fn cannot_down_log() -> Vec<TextFragment> {
		vec![TextFragment::new("そこからは降りられない。").color(
			Color::new(0.5, 0.5, 0.5, 1.0)
		)]
	}

	pub fn try_get_but_nothing_log() -> Vec<TextFragment> {
		vec![TextFragment::new("そこには何もない。").color(
				Color::new(0.5, 0.5, 0.5, 1.0))]
	}

	pub fn heal_log(name: &String, amount:i32) -> Vec<TextFragment> {
		vec![TextFragment::new(name.to_string()).color(
				Color::new(1.0, 1.0, 0.0, 1.0)),
			TextFragment::new("を使った。HPが").color(
				Color::new(1.0, 1.0, 1.0, 1.0)),
			TextFragment::new(amount.to_string()).color(
				Color::new(1.0, 1.0, 0.0, 1.0)),
			TextFragment::new("回復した。").color(
				Color::new(1.0, 1.0, 1.0, 1.0)),]	
	}

	pub fn try_do_item_but_no_item() -> Vec<TextFragment> {
		vec![TextFragment::new("そのようなアイテムを持っていない。").color(
				Color::new(0.5, 0.5, 0.5, 1.0))]
	}
	
	pub fn battle_log(name : &String, target_name: &String, damage: i32) -> Vec<TextFragment> {
		vec![
			TextFragment::new(name.to_string()).color(Color::new(1.0, 1.0, 0.0, 1.0)),
			TextFragment::new("は"),
			TextFragment::new(target_name.to_string()).color(Color::new(1.0, 1.0, 0.0, 1.0)),
			TextFragment::new("に"),
			TextFragment::new(damage.to_string()).color(Color::new(1.0, 0.0, 0.0, 1.0)),
			TextFragment::new("のダメージを与えた。")
		]
	}

	pub fn paralyze_log(name: &String) -> Vec<TextFragment> {
		vec![
			TextFragment::new(name.to_string()).color(NAME_COLOR),
			TextFragment::new("は"),
			TextFragment::new("麻痺").color(Color::new(0.5, 0.5, 1.0, 1.0)),
			TextFragment::new("した。")
		]
	}

	pub fn goto_next_level_log() -> Vec<TextFragment> {
		vec![TextFragment::new("あなたは階段を下ることにした……。")]
	}

}