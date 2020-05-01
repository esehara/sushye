use super::{CombatStats, DurationTurnHeal};
use specs::prelude::*;
use std::cmp::min;

pub struct TurnHealing {}
const DURATION_TURNHEAL: i32 = 10;

fn turn_heal_amount() -> i32 {
    1
}

impl<'a> System<'a> for TurnHealing {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, CombatStats>,
        WriteStorage<'a, DurationTurnHeal>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut combat_stats, mut turnheals) = data;
        for (_entity, stats, turnheal) in (&entities, &mut combat_stats, &mut turnheals).join() {
            turnheal.time += 1;
            if turnheal.time >= DURATION_TURNHEAL && stats.hp > 0 {
                stats.hp = min(stats.max_hp, stats.hp + turn_heal_amount());
                turnheal.time = 0;
            }
        }
    }
}
