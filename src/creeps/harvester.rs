use super::{Creep, Mode, ModeFlow};
use screeps::{prelude::*, Part};

pub const NAME_PREFIX: &'static str = "harvester";
pub struct Harvester;

impl ModeFlow for Harvester {
    fn get_new_mode(&self, creep: &Creep) -> Option<Mode> {
        if creep.creep.store_free_capacity(None) == 0 {
            return Some(Mode::TransferTo);
        } else if creep.creep.store_used_capacity(None) == 0 {
            return Some(Mode::Harvest);
        }

        return None;
    }
    fn consumtpion_rate(&self, creep: &Creep) -> u32 {
        return screeps::constants::HARVEST_POWER * creep.creep.get_active_bodyparts(Part::Work);
    }
}
