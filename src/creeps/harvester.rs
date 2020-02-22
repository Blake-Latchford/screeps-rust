use super::{Creep, Mode};
use log::*;
use screeps::{find, prelude::*, Part, RawObjectId, Source};

pub const NAME_PREFIX: &'static str = "harvester";
pub struct Harvester(screeps::Creep);

impl super::Creep for Harvester {
    fn get_creep(&self) -> &screeps::Creep {
        return &self.0;
    }

    fn update_mode(&self) {
        if self.get_creep().store_free_capacity(None) == 0 {
            self.set_mode(Mode::TransferTo);
        } else if self.get_creep().store_used_capacity(None) == 0 {
            self.set_mode(Mode::Harvest);
        }
    }

    fn update_target(&self) {
        if let Some(mode) = self.get_mode() {
            let target_id = match mode {
                Mode::TransferTo => self.get_transfer_target(),
                Mode::Harvest => self.get_harvest_target(),
                _ => None,
            };
            self.set_target(target_id);
        } else {
            warn!("No mode selected.");
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

    fn get_harvest_target(&self) -> Option<RawObjectId> {
        self.get_stored_id("harvest")
    }

    fn set_harvest_target_source(&self, source: screeps::Source) {
        debug!("setting new source: {:?}", source.id());
        self.get_creep()
            .memory()
            .set("harvest", source.id().to_string());
    }

    fn get_transfer_target(&self) -> Option<RawObjectId> {
        Some(screeps::game::spawns::values().pop()?.untyped_id())
    }
}

pub struct HarvesterManager {
    pub harvesters: Vec<Harvester>,
}

impl HarvesterManager {
    pub fn default() -> HarvesterManager {
        HarvesterManager {
            harvesters: Vec::new(),
        }
    }

    pub fn register(&mut self, creep: screeps::Creep) {
        self.harvesters.push(Harvester(creep));
    }

    pub fn game_loop(&self) {
        if let Some(target_source) = self.get_target_source() {
            for harvester in &self.harvesters {
                if harvester.get_harvest_target().is_none() {
                    harvester.set_harvest_target_source(target_source);
                    break;
                }
            }
        }

        for harvester in &self.harvesters {
            harvester.game_loop();
        }
    }

    pub fn get_target_source(&self) -> Option<Source> {
        let mut sources = Vec::new();
        for room in screeps::game::rooms::values() {
            if let Some(controller) = room.controller() {
                if controller.my() {
                    sources.extend(room.find(find::SOURCES));
                }
            }
        }
        for harvester in &self.harvesters {
            if let Some(harvest_target_id) = harvester.get_harvest_target() {
                if let Some(index) = sources
                    .iter()
                    .position(|x| x.untyped_id() == harvest_target_id)
                {
                    sources.remove(index);
                }
            }
        }
        if let Some(target_source) = sources.last() {
            debug!("Found untargeted source: {:?}", target_source.id());
        }
        return sources.pop();
    }
}
