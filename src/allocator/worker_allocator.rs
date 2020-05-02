use log::*;
use screeps::{
    find, prelude::*, ConstructionSite, Part, Position, RawObjectId, StructureController,
};

use crate::creeps::Creep;

pub const NAME_PREFIX: &'static str = "worker";

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

pub fn allocate_creep(creep: Creep) {
    allocate_input(&creep);
    allocate_output(&creep);
}

pub fn can_allocate_more() -> bool {
    let worker_count = screeps::game::creeps::values()
        .iter()
        .filter(|x| x.name().starts_with(NAME_PREFIX))
        .count();
    return worker_count < 2;
}

fn allocate_input(creep: &Creep) {
    if let Some(spawn) = screeps::game::spawns::values().pop() {
        creep.set_input(spawn.untyped_id());
    }
}

fn allocate_output(creep: &Creep) {
    if let Some(output_id) = allocate_output_id() {
        creep.set_output(output_id);
    }
}

fn allocate_output_id() -> Option<RawObjectId> {
    if let Some(controller_to_upgrade) = find_controller_to_upgrade() {
        return Some(controller_to_upgrade.untyped_id());
    }
    if let Some(last_site) = screeps::game::construction_sites::values().pop() {
        return Some(last_site.untyped_id());
    }
    return make_new_construction_site();
}

fn find_controller_to_upgrade() -> Option<StructureController> {
    const MINIMUM_DOWNGRADE_TIME: u32 = 5000;

    for controller in find_my_controllers() {
        if controller.level() <= 1 {
            return Some(controller);
        }
        if controller.ticks_to_downgrade() < MINIMUM_DOWNGRADE_TIME {
            return Some(controller);
        }
    }
    return None;
}

fn find_my_controllers() -> Vec<StructureController> {
    let mut result = vec![];
    for room in screeps::game::rooms::values() {
        if let Some(controller) = room.controller() {
            if controller.my() {
                result.push(controller);
            }
        }
    }
    return result;
}

fn make_new_construction_site() -> Option<RawObjectId> {
    if let Some(extension) = make_new_extension() {
        info!("New extension {:?}", extension.pos());
        return Some(extension.untyped_id());
    }

    return None;
}

fn make_new_extension() -> Option<ConstructionSite> {
    for controller in find_my_controllers() {
        if can_build_extension(&controller) {
            return place_extension_construction_site(&controller);
        }
    }

    return None;
}

fn can_build_extension(controller: &StructureController) -> bool {
    let current_nubmer_of_extensions = count_extensions(controller);
    let max_extensions = get_max_number_of_extensions(controller);

    debug!(
        "Extensions: {}/{}",
        current_nubmer_of_extensions, max_extensions
    );
    return current_nubmer_of_extensions < max_extensions;
}

fn get_max_number_of_extensions(controller: &StructureController) -> usize {
    return match controller.level() {
        2 => 5,
        3 => 10,
        4 => 20,
        5 => 30,
        6 => 40,
        7 => 50,
        8 => 60,
        _ => 0,
    };
}

fn place_extension_construction_site(controller: &StructureController) -> Option<ConstructionSite> {
    let max_extensions = get_max_number_of_extensions(controller);
    let extension_root = find_extension_root(controller)?;
    let mut extension_position_index = 1;

    assert!(max_extensions > 0);

    loop {
        // Skip the center of the spiral because the spawn is there.
        // Add gaps for roads.
        let extension_spiral_index = (2 * extension_position_index) + 1;
        let extension_position =
            get_position_at_spiral_index(extension_root, extension_spiral_index);

        let return_code = controller
            .room()
            .create_construction_site(&extension_position, screeps::StructureType::Extension);
        if return_code == screeps::ReturnCode::Ok {
            return controller
                .room()
                .look_for_at(screeps::look::CONSTRUCTION_SITES, &extension_position)
                .pop();
        }
        extension_position_index += 1;
    }
}

fn count_extensions(controller: &StructureController) -> usize {
    return controller
        .room()
        .find(find::MY_STRUCTURES)
        .iter()
        .filter(|x| x.structure_type() == screeps::StructureType::Extension)
        .count();
}

fn find_extension_root(controller: &StructureController) -> Option<Position> {
    //max_by_key to guarantee deterministic result.
    return Some(
        controller
            .room()
            .find(find::MY_SPAWNS)
            .iter()
            .max_by_key(|x| x.name())?
            .pos(),
    );
}

fn get_position_at_spiral_index(origin: Position, index: usize) -> Position {
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
