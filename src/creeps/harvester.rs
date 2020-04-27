use super::{Creep, Mode};
use log::*;
use screeps::{prelude::*, Part, RawObjectId};

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
    fn get_harvest_target(&self) -> Option<RawObjectId> {
        return self.get_stored_id("harvest");
    }

    pub fn set_harvest_target_source(&self, source: screeps::Source) {
        debug!("setting new source: {:?}", source.id());
        self.get_creep()
            .memory()
            .set("harvest", source.id().to_string());
    }

    fn get_transfer_target(&self) -> Option<RawObjectId> {
        Some(screeps::game::spawns::values().pop()?.untyped_id())
    }

    pub fn consumtpion_rate(&self) -> u32 {
        return screeps::constants::HARVEST_POWER
            * self.get_creep().get_active_bodyparts(Part::Work);
    }
}
