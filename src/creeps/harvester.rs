use log::*;
use screeps::{find, prelude::*, ObjectId, Part, ResourceType, ReturnCode};

pub const NAME_PREFIX: &'static str = "harvester";

pub struct Harvester(screeps::Creep);

impl super::Creep for Harvester {
    fn new(creep: screeps::Creep) -> Self {
        Harvester(creep)
    }

    fn game_loop(self) {
        let name = self.0.name();
        debug!("running creep {}", name);
        if self.0.spawning() {
            return;
        }

        if self.0.store_free_capacity(None) == 0 {
            self.carry_energy();
        } else {
            self.harvest();
        }
    }
}

impl Harvester {
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

    fn carry_energy(&self) {
        let spawns = self.0.room().find(find::MY_SPAWNS);
        if spawns.len() == 0 {
            warn!("creep room has no spawn.");
            return;
        }

        let spawn = &spawns[0];
        if self.0.pos().is_near_to(spawn) {
            self.0.transfer_all(spawn, ResourceType::Energy);
        } else {
            self.0.move_to(spawn);
        }
    }

    fn harvest(&self) {
        if let Some(source) = self.get_target() {
            if self.0.pos().is_near_to(&source) {
                let r = self.0.harvest(&source);
                if r != ReturnCode::Ok {
                    warn!("couldn't harvest: {:?}", r);
                }
            } else {
                self.0.move_to(&source);
            }
        }
    }

    fn get_target(&self) -> Option<screeps::Source> {
        if let Ok(Some(target_string)) = self.0.memory().string("target") {
            let target_id: ObjectId<screeps::Source> = target_string.parse().ok()?;
            return screeps::game::get_object_typed(target_id).ok()?;
        }

        if let Ok(Some(target_string)) = screeps::memory::root().string("target") {
            self.0.memory().set("target", &target_string);
            screeps::memory::root().del("target");
            let target_id: ObjectId<screeps::Source> = target_string.parse().ok()?;
            return screeps::game::get_object_typed(target_id).ok()?;
        }

        None
    }
}
