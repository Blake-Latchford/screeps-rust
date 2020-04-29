use super::Creep;
use super::Mode;
use screeps::{prelude::*, ConstructionSite};
pub const NAME_PREFIX: &'static str = "worker";

pub struct Worker(pub screeps::Creep);

impl Creep for Worker {
    fn get_creep(&self) -> &screeps::Creep {
        return &self.0;
    }

    fn get_new_mode(&self) -> Option<Mode> {
        if self.should_start_upgrade() {
            return Some(Mode::UpgradeController);
        } else if self.should_start_build() {
            return Some(Mode::Build);
        } else if self.should_start_transfer_from() {
            return Some(Mode::TransferFrom);
        } else if self.should_start_idle() {
            return Some(Mode::Idle);
        }

        return None;
    }
}

impl Worker {
    fn should_start_upgrade(&self) -> bool {
        if self.get_mode() != Mode::Idle && self.has_capacity() {
            return false;
        }

        if let Some(controller) = self.get_creep().room().controller() {
            if controller.level() <= 1 {
                return true;
            }
            if controller.ticks_to_downgrade() < 5000 {
                return true;
            }
        }
        return false;
    }

    fn should_start_transfer_from(&self) -> bool {
        if self.get_mode() != Mode::Idle && !self.is_empty() {
            return false;
        }

        if let Some(spawn) = screeps::game::spawns::values().pop() {
            if spawn.store_free_capacity(Some(screeps::ResourceType::Energy)) == 0 {
                return true;
            }
        }

        return false;
    }

    fn should_start_build(&self) -> bool {
        if self.is_full()
            && self
                .get_stored_object::<ConstructionSite>("output")
                .is_some()
        {
            return true;
        }

        return false;
    }

    fn should_start_idle(&self) -> bool {
        if let Some(spawn) = screeps::game::spawns::values().pop() {
            if spawn.store_free_capacity(None) != 0 {
                return true;
            }
        }

        return false;
    }
}
