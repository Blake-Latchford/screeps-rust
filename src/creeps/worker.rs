use super::Creep;
use super::Mode;
use screeps::{prelude::*, RawObjectId};
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

    fn get_new_target(&self) -> Option<RawObjectId> {
        return match self.get_mode() {
            Mode::UpgradeController => self.get_upgrade_controller_target(),
            Mode::TransferFrom => self.get_transfer_from_target(),
            Mode::Build => self.get_build_target(),
            Mode::Idle => self.get_idle_target(),
            _ => None,
        };
    }
}

impl Worker {
    fn should_start_upgrade(&self) -> bool {
        if !self.is_mode(Mode::Idle) && self.has_capacity() {
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
        if !self.is_mode(Mode::Idle) && !self.is_empty() {
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
        if self.is_full() && self.get_build_target().is_some() {
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

    fn get_upgrade_controller_target(&self) -> Option<RawObjectId> {
        Some(
            self.get_stored_object::<screeps::StructureController>("output")?
                .untyped_id(),
        )
    }

    fn get_transfer_from_target(&self) -> Option<RawObjectId> {
        Some(
            self.get_stored_object::<screeps::StructureSpawn>("input")?
                .untyped_id(),
        )
    }

    fn get_build_target(&self) -> Option<RawObjectId> {
        Some(
            self.get_stored_object::<screeps::ConstructionSite>("output")?
                .untyped_id(),
        )
    }

    fn get_idle_target(&self) -> Option<RawObjectId> {
        Some(
            self.get_stored_object::<screeps::StructureSpawn>("input")?
                .untyped_id(),
        )
    }
}
