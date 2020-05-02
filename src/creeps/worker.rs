use super::{Creep, Mode, ModeFlow};
use screeps::{prelude::*, ConstructionSite, Part};
pub const NAME_PREFIX: &'static str = "worker";

pub struct Worker;

impl ModeFlow for Worker {
    fn get_new_mode(&self, creep: &Creep) -> Option<Mode> {
        if self.should_start_upgrade(creep) {
            return Some(Mode::UpgradeController);
        } else if self.should_start_build(creep) {
            return Some(Mode::Build);
        } else if self.should_start_transfer_from(creep) {
            return Some(Mode::TransferFrom);
        } else if self.should_start_idle(creep) {
            return Some(Mode::Idle);
        }

        return None;
    }

    fn consumtpion_rate(&self, creep: &Creep) -> u32 {
        return screeps::constants::BUILD_POWER * creep.creep.get_active_bodyparts(Part::Work);
    }
}
impl Worker {
    fn should_start_upgrade(&self, creep: &Creep) -> bool {
        if creep.get_mode() != Mode::Idle && creep.has_capacity() {
            return false;
        }

        if let Some(controller) = creep.creep.room().controller() {
            if controller.level() <= 1 {
                return true;
            }
            if controller.ticks_to_downgrade() < 5000 {
                return true;
            }
        }
        return false;
    }

    fn should_start_transfer_from(&self, creep: &Creep) -> bool {
        if creep.get_mode() != Mode::Idle && !creep.is_empty() {
            return false;
        }

        if let Some(spawn) = screeps::game::spawns::values().pop() {
            if spawn.store_free_capacity(Some(screeps::ResourceType::Energy)) == 0 {
                return true;
            }
        }

        return false;
    }

    fn should_start_build(&self, creep: &Creep) -> bool {
        if creep.is_full()
            && creep
                .get_stored_object::<ConstructionSite>("output")
                .is_some()
        {
            return true;
        }

        return false;
    }

    fn should_start_idle(&self, _: &Creep) -> bool {
        if let Some(spawn) = screeps::game::spawns::values().pop() {
            if spawn.store_free_capacity(None) != 0 {
                return true;
            }
        }

        return false;
    }
}
