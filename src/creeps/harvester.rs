use super::{Creep, Mode};
use log::*;
use screeps::{find, prelude::*, Part, RawObjectId, Source};
use std::collections::HashMap;
use std::convert::TryInto;

pub const NAME_PREFIX: &'static str = "harvester";
pub struct Harvester(pub screeps::Creep);

impl super::Creep for Harvester {
    fn get_creep(&self) -> &screeps::Creep {
        return &self.0;
    }

    fn get_new_mode(&self) -> Option<Mode> {
        if self.get_creep().store_free_capacity(None) == 0 {
            return Some(Mode::TransferTo);
        } else if self.get_creep().store_used_capacity(None) == 0 {
            return Some(Mode::Harvest);
        }

        return None;
    }

    fn get_new_target(&self) -> Option<RawObjectId> {
        if let Some(mode) = self.get_mode() {
            return match mode {
                Mode::TransferTo => self.get_transfer_target(),
                Mode::Harvest => self.get_harvest_target(),
                _ => None,
            };
        }
        warn!("No mode selected.");
        return None;
    }
}

impl Harvester {
    pub fn get_description(capacity: u32) -> (Vec<Part>, &'static str) {
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

        (body, NAME_PREFIX)
    }

    fn get_harvest_target(&self) -> Option<RawObjectId> {
        let stored_harvest_target = self.get_stored_id("harvest");
        if stored_harvest_target.is_none() {
            debug!("{} has no target", self.get_creep().name());
            if let Some(target_source) = Harvester::get_target_source() {
                info!(
                    "Allocateed source {} to {}",
                    target_source.id(),
                    self.get_creep().name()
                );
                self.set_harvest_target_source(target_source);
                return self.get_stored_id("harvest");
            }
        }
        return stored_harvest_target;
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

    pub fn get_target_source() -> Option<Source> {
        let mut harvesters = vec![];
        for creep in screeps::game::creeps::values() {
            if creep.name().starts_with(NAME_PREFIX) {
                harvesters.push(Harvester(creep));
            }
        }

        return Harvester::get_source_with_most_capacity(&harvesters);
    }

    fn get_source_with_most_capacity(harvesters: &Vec<Harvester>) -> Option<Source> {
        let source_id = Harvester::source_creep_map(harvesters)
            .drain()
            .max_by_key(|(k, v)| Harvester::wasted_input_rate(&k, &v))?
            .0;
        debug!("{}:{}", file!(), line!());
        return screeps::game::get_object_typed::<Source>(source_id.into()).ok()?;
    }

    fn source_creep_map(harvesters: &Vec<Harvester>) -> HashMap<RawObjectId, Vec<&Harvester>> {
        let mut result = HashMap::new();

        for source in Harvester::get_my_sources() {
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

    fn wasted_input_rate(source_id: &RawObjectId, harvesters: &Vec<&Harvester>) -> i32 {
        let source = screeps::game::get_object_typed::<Source>((*source_id).into())
            .unwrap()
            .unwrap();
        let input_rate: i32 = Harvester::input_rate(source).try_into().unwrap();
        let output_rate: i32 = Harvester::output_rate(harvesters).try_into().unwrap();

        return input_rate - output_rate;
    }

    fn input_rate(source: Source) -> u32 {
        return source.energy_capacity() / screeps::constants::ENERGY_REGEN_TIME;
    }

    fn output_rate(harvesters: &Vec<&Harvester>) -> u32 {
        return harvesters.iter().map(|h| h.consumtpion_rate()).sum();
    }
}
