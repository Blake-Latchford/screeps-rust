use log::*;
use screeps::{find, prelude::*, Part, ResourceType, ReturnCode};

const name_prefix: &'static str = "starter";
pub fn get_description() -> (Vec<Part>, &'static str) {
    (
        vec![Part::Move, Part::Move, Part::Carry, Part::Work],
        name_prefix,
    )
}

pub fn game_loop(creep: screeps::Creep) {
    let name = &creep.name();
    debug!("running creep {}", name);
    if creep.spawning() {
        return;
    }

    if creep.memory().bool("harvesting") {
        if creep.store_free_capacity(Some(ResourceType::Energy)) == 0 {
            creep.memory().set("harvesting", false);
        }
    } else {
        if creep.store_used_capacity(None) == 0 {
            creep.memory().set("harvesting", true);
        }
    }

    if creep.memory().bool("harvesting") {
        let source = &creep.room().find(find::SOURCES)[0];
        if creep.pos().is_near_to(source) {
            let r = creep.harvest(source);
            if r != ReturnCode::Ok {
                warn!("couldn't harvest: {:?}", r);
            }
        } else {
            creep.move_to(source);
        }
    } else {
        if let Some(c) = creep.room().controller() {
            let r = creep.upgrade_controller(&c);
            if r == ReturnCode::NotInRange {
                creep.move_to(&c);
            } else if r != ReturnCode::Ok {
                warn!("couldn't upgrade: {:?}", r);
            }
        } else {
            warn!("creep room has no controller!");
        }
    }
}

pub fn name_matches(name: &String) -> bool {
    name.starts_with(name_prefix);
}
