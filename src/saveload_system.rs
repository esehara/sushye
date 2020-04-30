use specs::prelude::*;
use specs::saveload::{SimpleMarker, SimpleMarkerAllocator, SerializeComponents, DeserializeComponents, MarkedBuilder};
use specs::error::NoError;
use super::*;
use std::fs::File;
use std::path::Path;
use std::fs;

macro_rules! serialize_individually {
    ($ecs:expr, $ser:expr, $data:expr, $( $type:ty),*) => {
        $(
        SerializeComponents::<NoError, SimpleMarker<SerializeMe>>::serialize(
            &( $ecs.read_storage::<$type>(), ),
            &$data.0,
            &$data.1,
            &mut $ser,
        )
        .unwrap();
        )*
    };
}

pub fn save_game(ecs : &mut World) {
    // Create helper
    let mapcopy = ecs.get_mut::<super::map::Map>().unwrap().clone();
    let savehelper = ecs
        .create_entity()
        .with(SerializationHelper{ map : mapcopy })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();

    // Actually serialize
    {
        let data = ( ecs.entities(), ecs.read_storage::<SimpleMarker<SerializeMe>>() );

        let writer = File::create("./savegame.json").unwrap();
        let mut serializer = serde_json::Serializer::new(writer);
        serialize_individually!(ecs, serializer, data, Position, Renderable, Player, Viewshed, Monster, 
            Name, BlocksTile, CombatStats, SufferDamage, WantsToMelee, Item, Paralyze, Ranged, InflictsDamage, 
            AreaOfEffect, Paralyze, ProvidesHealing, InBackpack, WantsToPickupItem, WantsToUseItem,
			DropItem, SerializationHelper);
    }

    // Clean up
    ecs.delete_entity(savehelper).expect("Crash on cleanup");
}