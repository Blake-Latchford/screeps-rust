use log::*;
use screeps::{find, prelude::*, ObjectId, Part, ResourceType, ReturnCode};

pub fn game_loop() {
    for spawn in screeps::game::spawns::values() {
        debug!("running spawn {}", spawn.name());

        if let Some(_) = spawn.spawning() {
            continue;
        }

        if let Some((body, name_prefix)) = get_spawn_target(&spawn) {
            let spawn_cost = body.iter().map(|p| p.cost()).sum();
            if spawn.energy() >= spawn_cost {
                spawn_creep(spawn, body, name_prefix);
            }
        }
    }
}

fn get_spawn_target(spawn: &screeps::StructureSpawn) -> Option<(Vec<Part>, &'static str)> {
    if has_next_target_source(spawn) {
        return Some(super::creeps::harvester::get_description(
            spawn.store_capacity(Some(ResourceType::Energy)),
        ));
    }

    return None;
}

fn has_next_target_source(spawn: &screeps::StructureSpawn) -> bool {
    let mut sources = spawn.room().find(find::SOURCES);

    if let Some(_) = get_target(screeps::memory::root()) {
        return true;
    }

    for creep in screeps::game::creeps::values() {
        if let Some(creep_target) = get_creep_target(creep) {
            if let Some(index) = sources.iter().position(|x| *x == creep_target) {
                sources.remove(index);
            }
        }
    }
    sources.sort_by_key(|s| s.pos().get_range_to(&spawn.pos()));

    if sources.len() > 0 {
        screeps::memory::root().set("target", sources[0].id().to_string());
        return true;
    }

    return false;
}

fn get_creep_target(creep: screeps::Creep) -> Option<screeps::Source> {
    get_target(creep.memory())
}

fn get_target(mem: screeps::memory::MemoryReference) -> Option<screeps::Source> {
    let target_string = mem.string("target").ok()??;
    let target_id: ObjectId<screeps::Source> = target_string.parse().ok()?;
    screeps::game::get_object_typed(target_id).ok()?
}

fn spawn_creep(spawn: screeps::StructureSpawn, body: Vec<screeps::Part>, name_prefix: &str) {
    for i in 0..1000 {
        let name = name_prefix.to_owned() + &i.to_string();
        let return_code = spawn.spawn_creep(&body, &name);
        match return_code {
            ReturnCode::NameExists => continue,
            ReturnCode::Ok => return,
            _ => warn!("couldn't spawn: {:?}", return_code),
        }
        return;
    }
}
