use super::creeps;
use screeps::Part;

mod harvester_allocator;
mod worker_allocator;

pub fn allocate_creeps() {
    for creep in screeps::game::creeps::values() {
        allocate_creep(creeps::Creep::new(creep));
    }
}

fn allocate_creep(creep: creeps::Creep) {
    match creep.role {
        creeps::Role::Harvester => harvester_allocator::allocate_creep(creep),
        creeps::Role::Worker => worker_allocator::allocate_creep(creep),
    }
}

pub fn get_spawn_target(capacity: u32) -> Option<(Vec<Part>, creeps::Role)> {
    const DESCRIPTORS: [(creeps::Role, &dyn Fn(u32) -> Option<Vec<Part>>); 2] = [
        (
            creeps::Role::Harvester,
            &harvester_allocator::get_description,
        ),
        (creeps::Role::Worker, &worker_allocator::get_description),
    ];
    for (role, description_func) in DESCRIPTORS.iter() {
        if let Some(description) = description_func(capacity) {
            return Some((description, role.clone()));
        }
    }

    None
}
