use super::{Creep, Mode};
use screeps::{prelude::*, Part};

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
}

impl Harvester {
    pub fn consumtpion_rate(&self) -> u32 {
        return screeps::constants::HARVEST_POWER
            * self.get_creep().get_active_bodyparts(Part::Work);
    }
}
