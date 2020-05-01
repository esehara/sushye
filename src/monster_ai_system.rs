use super::{Map, Monster, Paralyze, Position, RunState, SeenPlayer, Viewshed, WantsToMelee};
use bracket_lib::prelude::{a_star_search, DistanceAlg, Point};
use specs::prelude::*;

pub struct MonsterAI {}

impl<'a> System<'a> for MonsterAI {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadExpect<'a, Point>,
        ReadExpect<'a, Entity>,
        ReadExpect<'a, RunState>,
        Entities<'a>,
        WriteStorage<'a, Viewshed>,
        WriteStorage<'a, SeenPlayer>,
        ReadStorage<'a, Monster>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, Paralyze>,
        WriteStorage<'a, WantsToMelee>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut map,
            player_pos,
            player_entity,
            runstate,
            entities,
            mut viewshed,
            mut seenplayers,
            monster,
            mut position,
            mut paralyze,
            mut wants_to_melee,
        ) = data;

        if *runstate != RunState::MonsterTurn {
            return;
        }
        for (entity, mut viewshed, mut seenplayer, _monster, mut pos) in (
            &entities,
            &mut viewshed,
            &mut seenplayers,
            &monster,
            &mut position,
        )
            .join()
        {
            let mut can_act = true;
            let is_paralyze = paralyze.get_mut(entity);
            match is_paralyze {
                None => {}
                Some(i_am_paralyze) => {
                    i_am_paralyze.turns -= 1;
                    if i_am_paralyze.turns < 1 {
                        paralyze.remove(entity);
                    }
                    can_act = false;
                }
            }

            if viewshed.visible_tiles.contains(&*player_pos) {
                seenplayer.point = Some(*player_pos);
            }

            if can_act {
                match seenplayer.point {
                    None => {}
                    Some(seen_player_pos) => {
                        let distance = DistanceAlg::Pythagoras
                            .distance2d(Point::new(pos.x, pos.y), *player_pos);
                        if distance < 1.5 {
                            wants_to_melee
                                .insert(
                                    entity,
                                    WantsToMelee {
                                        target: *player_entity,
                                    },
                                )
                                .expect("Unable to insert attack");
                        } else {
                            let path = a_star_search(
                                map.xy_idx(pos.x, pos.y),
                                map.xy_idx(seen_player_pos.x, seen_player_pos.y),
                                &mut *map,
                            );
                            if path.success && path.steps.len() > 1 {
                                let mut idx = map.xy_idx(pos.x, pos.y);
                                map.blocked[idx] = false;
                                pos.x = path.steps[1] as i32 % map.width;
                                pos.y = path.steps[1] as i32 / map.width;
                                idx = map.xy_idx(pos.x, pos.y);
                                map.blocked[idx] = true;
                                viewshed.dirty = true;
                            }
                        }

                        if seen_player_pos.x == pos.x && seen_player_pos.y == pos.y {
                            seenplayer.point = None;
                        }
                    }
                }
            }
        }
    }
}
