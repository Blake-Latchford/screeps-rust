use log::*;
use screeps::{find, prelude::*, ObjectId, Part, ResourceType, ReturnCode};

const NAME_PREFIX: &'static str = "harvester";
pub fn get_description(capacity: u32) -> (Vec<Part>, &'static str) {
    let mut body = vec![Part::Move, Part::Carry];
    let remaining_cost = capacity - body.iter().map(|p| p.cost()).sum::<u32>();
    let extra_work_parts = remaining_cost / Part::Work.cost();
    let extra_carry_parts =
        (remaining_cost - (extra_work_parts * Part::Work.cost())) / Part::Carry.cost();

    for _ in 0..extra_work_parts {
        body.push(Part::Work);
    }

    for _ in 0..extra_carry_parts {
        body.push(Part::Carry);
    }

    (body, NAME_PREFIX)
}

pub fn name_matches(name: &String) -> bool {
    name.starts_with(NAME_PREFIX)
}

pub fn game_loop(creep: screeps::Creep) {
    let name = &creep.name();
    debug!("running creep {}", name);
    if creep.spawning() {
        return;
    }

    if creep.store_free_capacity(None) == 0 {
        carry_energy(creep);
    } else {
        harvest(creep);
    }
}

fn carry_energy(creep: screeps::Creep) {
    let spawns = creep.room().find(find::MY_SPAWNS);
    if spawns.len() == 0 {
        warn!("creep room has no spawn.");
        return;
    }

    let spawn = &spawns[0];
    if creep.pos().is_near_to(spawn) {
        creep.transfer_all(spawn, ResourceType::Energy);
    } else {
        creep.move_to(spawn);
    }
}

fn harvest(creep: screeps::Creep) {
    if let Some(source) = get_target(&creep) {
        if creep.pos().is_near_to(&source) {
            let r = creep.harvest(&source);
            if r != ReturnCode::Ok {
                warn!("couldn't harvest: {:?}", r);
            }
        } else {
            creep.move_to(&source);
        }
    }
}

fn get_target(creep: &screeps::Creep) -> Option<screeps::Source> {
    if let Ok(Some(target_string)) = creep.memory().string("target") {
        let target_id: ObjectId<screeps::Source> = target_string.parse().ok()?;
        return screeps::game::get_object_typed(target_id).ok()?;
    }

    if let Ok(Some(target_string)) = screeps::memory::root().string("target") {
        creep.memory().set("target", &target_string);
        screeps::memory::root().del("target");
        let target_id: ObjectId<screeps::Source> = target_string.parse().ok()?;
        return screeps::game::get_object_typed(target_id).ok()?;
    }

    None
}
