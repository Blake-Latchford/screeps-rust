use log::*;
use screeps::{find, prelude::*, Part, ReturnCode};

const NAME_PREFIX: &'static str = "starter";
pub fn get_description() -> (Vec<Part>, &'static str) {
    (
        vec![Part::Move, Part::Move, Part::Carry, Part::Work],
        NAME_PREFIX,
    )
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

fn harvest(creep: screeps::Creep) {
    let spawn = &creep.room().find(find::MY_SPAWNS)[0];
    if let Some(source) = &spawn.pos().find_closest_by_range(find::SOURCES) {
        if creep.pos().is_near_to(source) {
            let r = creep.harvest(source);
            if r != ReturnCode::Ok {
                warn!("couldn't harvest: {:?}", r);
            }
        } else {
            creep.move_to(source);
        }
    }
}
