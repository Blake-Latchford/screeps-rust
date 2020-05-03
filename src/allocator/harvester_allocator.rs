use log::*;
use screeps::{find, prelude::*, HasId, Part, RawObjectId, Source};
use std::collections::HashMap;
use std::convert::TryInto;

use crate::creeps;

pub fn get_description(capacity: u32) -> Option<Vec<Part>> {
    if get_target_source().is_none() {
        return None;
    }

    let mut body = vec![Part::Move, Part::Carry];
    let base_body_cost = body.iter().map(|p| p.cost()).sum::<u32>();
    assert!(capacity >= base_body_cost);
    let remaining_capacity = capacity - base_body_cost;
    let extra_work_parts = remaining_capacity / Part::Work.cost();
    let extra_carry_parts =
        (remaining_capacity - (extra_work_parts * Part::Work.cost())) / Part::Carry.cost();

    for _ in 0..extra_work_parts {
        body.push(Part::Work);
    }

    for _ in 0..extra_carry_parts {
        body.push(Part::Carry);
    }

    Some(body)
}

pub fn allocate_creeps(creeps: Vec<creeps::Creep>) {
    for creep in creeps {
        allocate_creep(creep);
    }
}

fn allocate_creep(creep: creeps::Creep) {
    if creep.get_input::<Source>().is_none() {
        if let Some(target_source) = get_target_source() {
            creep.set_input(target_source.untyped_id());
        }
    }

    allocate_creep_output(creep);
}

fn get_target_source() -> Option<Source> {
    let mut harvesters = vec![];
    for screeps_creep in screeps::game::creeps::values() {
        let creep = creeps::Creep::new(screeps_creep);
        if creep.role == creeps::Role::Harvester {
            harvesters.push(creep);
        }
    }

    return get_source_with_most_capacity(harvesters);
}

fn get_source_with_most_capacity(creeps: Vec<creeps::Creep>) -> Option<Source> {
    let source_id = source_creep_map(creeps)
        .drain()
        .filter(|(k, v)| v.len() < max_creeps(k))
        .max_by_key(|(k, v)| wasted_input_rate(&k, v))?
        .0;
    debug!("{}:{}", file!(), line!());
    return screeps::game::get_object_typed::<Source>(source_id.into()).ok()?;
}

fn source_creep_map(creep: Vec<creeps::Creep>) -> HashMap<RawObjectId, Vec<creeps::Creep>> {
    let mut result = HashMap::new();

    for source in get_my_sources() {
        result.insert(source.untyped_id(), vec![]);
    }

    for harvester in creep {
        if let Some(source) = harvester.get_input::<Source>() {
            result
                .get_mut(&source.untyped_id())
                .unwrap()
                .push(harvester);
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

fn max_creeps(_source_id: &RawObjectId) -> usize {
    return 2;
}

fn wasted_input_rate(source_id: &RawObjectId, harvesters: &Vec<creeps::Creep>) -> i32 {
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

fn output_rate(harvesters: &Vec<creeps::Creep>) -> u32 {
    return harvesters.iter().map(|x| x.consumption_rate()).sum();
}

fn allocate_creep_output(creep: creeps::Creep) {
    if let Some(output) = creep.get_output::<screeps::Structure>() {
        if is_valid_output(&output) {
            return;
        }
    }

    let structures = screeps::game::structures::values();
    let mut available_stores: Vec<_> = structures.iter().filter(|x| is_valid_output(x)).collect();
    available_stores.sort_unstable_by_key(|x| creep.get_range_to(&x.pos()));
    if let Some(first) = available_stores.first() {
        creep.set_output(first.untyped_id());
    } else {
        warn!("Unable to find valid output.");
    }
}

fn is_valid_output(structure: &screeps::Structure) -> bool {
    if let Some(has_store) = structure.as_has_energy_for_spawn() {
        if has_store.store_free_capacity(None) <= 0 {
            return true;
        }
    }

    return false;
}
