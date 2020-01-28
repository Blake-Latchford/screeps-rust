use log::*;
use screeps::{prelude::*, Part, ReturnCode};

pub fn game_loop() {
    for spawn in screeps::game::spawns::values() {
        debug!("running spawn {}", spawn.name());

        if let Some(_) = spawn.spawning() {
            continue;
        }

        if let Some((body, name)) = get_spawn_target() {
            if spawn.energy() >= body.iter().map(|p| p.cost()).sum() {
                let response = spawn.spawn_creep(&body, &name);
                if response != ReturnCode::Ok {
                    warn!("couldn't spawn: {:?}", response);
                }
            }
        }
    }
}

fn get_spawn_target() -> Option<(Vec<Part>, String)> {
    let creeps = screeps::game::creeps::values();
    if creeps.len() == 0 {
        return Some((super::creeps::starter::get_body(), "starter".to_string()));
    }
    return None;
}
