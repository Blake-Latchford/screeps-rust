use super::Creep;
use super::Mode;
use log::*;
use screeps::{prelude::*, Part, RawObjectId};

pub const NAME_PREFIX: &'static str = "worker";

pub struct Worker(screeps::Creep);

impl Creep for Worker {
    fn get_creep(&self) -> &screeps::Creep {
        return &self.0;
    }

    fn update_mode(&self) {
        if self.get_creep().store_free_capacity(None) == 0 {
            self.set_mode(Mode::UpgradeController);
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
