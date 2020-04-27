use log::*;
use screeps::{find, prelude::*, RawObjectId, Source};
use std::collections::HashMap;
use std::convert::TryInto;

use crate::creeps::harvester::Harvester;
use crate::creeps::Creep;

pub const NAME_PREFIX: &'static str = "harvester";

pub fn allocate_creep(creep: screeps::Creep) {
    let harvester = Harvester(creep);
    if harvester.get_stored_id("harvest").is_none() {
        debug!("{} has no target", harvester.get_creep().name());
        if let Some(target_source) = get_target_source() {
            info!(
                "Allocateed source {} to {}",
                target_source.id(),
                harvester.get_creep().name()
            );
            harvester.set_harvest_target_source(target_source);
        }
    }
}

pub fn get_target_source() -> Option<Source> {
    let mut harvesters = vec![];
    for creep in screeps::game::creeps::values() {
        if creep.name().starts_with(NAME_PREFIX) {
            harvesters.push(Harvester(creep));
        }
    }

    return get_source_with_most_capacity(&harvesters);
}

fn get_source_with_most_capacity(harvesters: &Vec<Harvester>) -> Option<Source> {
    let source_id = source_creep_map(harvesters)
        .drain()
        .filter(|(k, v)| v.len() < source_capacity(k))
        .max_by_key(|(k, v)| wasted_input_rate(&k, &v))?
        .0;
    debug!("{}:{}", file!(), line!());
    return screeps::game::get_object_typed::<Source>(source_id.into()).ok()?;
}

fn source_creep_map(harvesters: &Vec<Harvester>) -> HashMap<RawObjectId, Vec<&Harvester>> {
    let mut result = HashMap::new();

    for source in get_my_sources() {
        result.insert(source.untyped_id(), vec![]);
    }

    for harvester in harvesters {
        if let Some(source_id) = harvester.get_stored_id("harvest") {
            result.get_mut(&source_id).unwrap().push(harvester);
        }
    }

    return result;
}

fn get_my_sources() -> Vec<Source> {
    let mut sources = Vec::new();
    for room in screeps::game::rooms::values() {
        if let Some(controller) = room.controller() {
            if controller.my() {
                sources.extend(room.find(find::SOURCES));
            }
        }
    }
    return sources;
}

fn source_capacty(source: &Source) {}

fn wasted_input_rate(source_id: &RawObjectId, harvesters: &Vec<&Harvester>) -> i32 {
    let source = screeps::game::get_object_typed::<Source>((*source_id).into())
        .unwrap()
        .unwrap();
    let input_rate: i32 = input_rate(source).try_into().unwrap();
    let output_rate: i32 = output_rate(harvesters).try_into().unwrap();

    return input_rate - output_rate;
}

fn input_rate(source: Source) -> u32 {
    return source.energy_capacity() / screeps::constants::ENERGY_REGEN_TIME;
}

fn output_rate(harvesters: &Vec<&Harvester>) -> u32 {
    return harvesters.iter().map(|h| h.consumtpion_rate()).sum();
}