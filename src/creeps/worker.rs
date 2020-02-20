use super::Creep;
use super::Mode;
use log::*;
use screeps::{find, prelude::*, ConstructionSite, Part, Position, RawObjectId};
pub const NAME_PREFIX: &'static str = "worker";

pub struct Worker(screeps::Creep);

impl Creep for Worker {
    fn get_creep(&self) -> &screeps::Creep {
        return &self.0;
    }

    fn update_mode(&self) {
        if self.get_creep().store_free_capacity(None) == 0 {
            debug!("Creep at capacity.");
            if self.should_upgrade() {
                self.set_mode(Mode::UpgradeController);
            } else if self.get_build_target().is_some() {
                debug!("Has build target.");
                self.set_mode(Mode::Build);
            } else {
                error!("No viable mode.");
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
                Mode::Build => self.get_build_target(),
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
            if controller.level() <= 1 {
                return true;
            }
            if controller.ticks_to_downgrade() < 5000 {
                return true;
            }
        }

        return false;
    }

    fn get_build_target(&self) -> Option<RawObjectId> {
        if let Some(last_site) = screeps::game::construction_sites::values().pop() {
            return Some(last_site.untyped_id());
        }

        return self.make_new_construction_site();
    }

    fn make_new_construction_site(&self) -> Option<RawObjectId> {
        if let Some(extension) = self.make_new_extension() {
            debug!("New extension {:?}", extension.pos());
            return Some(extension.untyped_id());
        }

        return None;
    }

    fn make_new_extension(&self) -> Option<ConstructionSite> {
        let my_structures = self.get_creep().room().find(find::MY_STRUCTURES);
        let current_nubmer_of_extensions = my_structures
            .iter()
            .filter(|x| x.structure_type() == screeps::StructureType::Extension)
            .count();

        let max_extensions = match self.get_creep().room().controller()?.level() {
            2 => 5,
            3 => 10,
            4 => 20,
            5 => 30,
            6 => 40,
            7 => 50,
            8 => 60,
            _ => 0,
        };

        if current_nubmer_of_extensions >= max_extensions {
            debug!(
                "No capacity for additional extensions: {}/{}",
                current_nubmer_of_extensions, max_extensions
            );
            return None;
        }

        // Leave a gap, and skip the center of the spiral because the spawn is there.
        let extension_position_index = (2 * current_nubmer_of_extensions) + 1;
        let root_spawn = screeps::game::spawns::values().pop()?;
        let new_extension_position =
            self.get_position_at_spiral_index(root_spawn.pos(), extension_position_index);

        let return_code = self
            .get_creep()
            .room()
            .create_construction_site(&new_extension_position, screeps::StructureType::Extension);
        if return_code != screeps::ReturnCode::Ok {
            debug!("Failed create_construction_site: {:?}", return_code);
            return None;
        }

        self.get_creep()
            .room()
            .look_for_at(screeps::look::CONSTRUCTION_SITES, &new_extension_position)
            .pop()
    }

    fn get_position_at_spiral_index(&self, origin: Position, index: usize) -> Position {
        if index <= 0 {
            return origin;
        }

        let radius_f64 = (((index as f64).sqrt() - 1.0) / 2.0) + 1.0;
        let radius: i32 = radius_f64 as i32;
        let sqrt_first_index_at_radius = (2 * radius) - 1;
        let first_index_at_radius = sqrt_first_index_at_radius * sqrt_first_index_at_radius;
        let ring_index: i32 = (index as i32) - first_index_at_radius;
        let ring_size = 8 * radius;
        let ring_side = (4 * ring_index) / ring_size;

        let ring_side_x_offet = match ring_side {
            0 => -radius,
            1 => radius,
            2 => -radius,
            3 => radius,
            _ => 0,
        };
        let ring_side_y_offet = match ring_side {
            0 => -radius,
            1 => -radius,
            2 => radius,
            3 => radius,
            _ => 0,
        };

        let ring_side_offset = ring_index - (ring_size * ring_side / 4);

        let ring_side_offset_x_offset: i32 = match ring_side {
            0 => ring_side_offset,
            2 => -ring_side_offset,
            _ => 0,
        };
        let ring_side_offset_y_offset = match ring_side {
            1 => ring_side_offset,
            3 => -ring_side_offset,
            _ => 0,
        };

        let x_offset = ring_side_x_offet + ring_side_offset_x_offset;
        let y_offset = ring_side_y_offet + ring_side_offset_y_offset;

        let mut result: Position = origin;
        result.offset(x_offset, y_offset);
        return result;
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
