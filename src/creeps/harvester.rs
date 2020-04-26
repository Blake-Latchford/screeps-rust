use super::{Creep, Mode};
use log::*;
use screeps::{find, prelude::*, Part, RawObjectId, Source};
use std::collections::HashMap;
use std::convert::TryInto;

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
        let base_body_cost = body.iter().map(|p| p.cost()).sum::<u32>();
        assert!(capacity >= base_body_cost);
        let remaining_cost = capacity - base_body_cost;
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

    fn consumtpion_rate(&self) -> u32 {
        return screeps::constants::HARVEST_POWER
            * self.get_creep().get_active_bodyparts(Part::Work);
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
        let source_id = HarvesterManager::source_creep_map(&self.harvesters)
            .drain()
            .max_by_key(|(k, v)| HarvesterManager::wasted_input_rate(&k, &v))?
            .0;
        debug!("{}:{}", file!(), line!());
        return screeps::game::get_object_typed::<Source>(source_id.into()).ok()?;
    }

    fn source_creep_map(harvesters: &Vec<Harvester>) -> HashMap<RawObjectId, Vec<&Harvester>> {
        let mut result = HashMap::new();

        for source in HarvesterManager::get_my_sources() {
            result.insert(source.untyped_id(), vec![]);
        }

        for harvester in harvesters {
            if let Some(source_id) = harvester.get_harvest_target() {
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

    fn wasted_input_rate(source_id: &RawObjectId, harvesters: &Vec<&Harvester>) -> i32 {
        let source = screeps::game::get_object_typed::<Source>((*source_id).into())
            .unwrap()
            .unwrap();
        let input_rate: i32 = HarvesterManager::input_rate(source).try_into().unwrap();
        let output_rate: i32 = HarvesterManager::output_rate(harvesters)
            .try_into()
            .unwrap();

        return input_rate - output_rate;
    }

    fn input_rate(source: Source) -> u32 {
        return source.energy_capacity() / screeps::constants::ENERGY_REGEN_TIME;
    }

    fn output_rate(harvesters: &Vec<&Harvester>) -> u32 {
        return harvesters.iter().map(|h| h.consumtpion_rate()).sum();
    }
}
