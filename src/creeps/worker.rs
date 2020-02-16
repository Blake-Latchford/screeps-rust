use super::Creep;
use super::Mode;
use log::*;
use screeps::{prelude::*, ConstructionSite, Part, RawObjectId};

pub const NAME_PREFIX: &'static str = "worker";

pub struct Worker(screeps::Creep);

impl Creep for Worker {
    fn get_creep(&self) -> &screeps::Creep {
        return &self.0;
    }

    fn update_mode(&self) {
        if self.get_creep().store_free_capacity(None) == 0 {
            if self.should_upgrade() {
                self.set_mode(Mode::UpgradeController);
            } else if self.get_build_target().is_some() {
                self.set_mode(Mode::Build);
            }
        } else if self.get_creep().store_used_capacity(None) == 0 {
            self.set_mode(Mode::TransferFrom);
        }
    }

    fn update_target(&self) {
        if let Some(mode) = self.get_mode() {
            let target_id = match mode {
                Mode::UpgradeController => self.get_upgrade_controller_target(),
                Mode::TransferFrom => self.get_transfer_from_target(),
                _ => None,
            };
            self.set_target(target_id);
        } else {
            warn!("No mode selected.")
        }
    }
}

impl Worker {
    pub fn get_description(capacity: u32) -> (Vec<Part>, &'static str) {
        let part_set = [Part::Move, Part::Carry, Part::Work];
        let part_set_cost: u32 = part_set.iter().map(|part| part.cost()).sum();
        let number_of_part_sets = capacity / part_set_cost;

        let mut result: Vec<Part> = Vec::new();
        for part in &part_set {
            for _ in 0..number_of_part_sets {
                result.push(*part);
            }
        }

        let mut left_over_energy = capacity - part_set_cost;
        for part in &part_set {
            if part.cost() <= left_over_energy {
                left_over_energy -= part.cost();
                result.push(*part);
            }
        }

        (result, NAME_PREFIX)
    }

    fn get_upgrade_controller_target(&self) -> Option<RawObjectId> {
        Some(self.get_creep().room().controller()?.untyped_id())
    }

    fn get_transfer_from_target(&self) -> Option<RawObjectId> {
        Some(screeps::game::spawns::values().pop()?.untyped_id())
    }

    fn should_upgrade(&self) -> bool {
        if let Some(controller) = self.get_creep().room().controller() {
            if controller.level() <= 0 {
                return true;
            }
            if controller.ticks_to_downgrade() < 5000 {
                return true;
            }
        }

        return false;
    }

    fn get_build_target(&self) -> Option<ConstructionSite> {
        let last_site = screeps::game::construction_sites::values().pop();
        if last_site.is_some() {
            return last_site;
        }

        return self.make_new_construction_site();
    }

    fn make_new_construction_site(&self) -> Option<ConstructionSite> {
        return None;
    }
}

pub struct WorkerManager {
    pub workers: Vec<Worker>,
}

impl WorkerManager {
    pub fn default() -> WorkerManager {
        WorkerManager {
            workers: Vec::new(),
        }
    }

    pub fn register(&mut self, creep: screeps::Creep) {
        self.workers.push(Worker(creep));
    }
}
