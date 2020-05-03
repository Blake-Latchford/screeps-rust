use super::creeps;
use screeps::Part;
use std::collections::HashMap;

mod harvester_allocator;
mod worker_allocator;

pub fn allocate_creeps() {
    let mut role_map = HashMap::new();
    for screeps_creep in screeps::game::creeps::values() {
        let creep = creeps::Creep::new(screeps_creep);
        role_map
            .entry(creep.role.clone())
            .or_insert(vec![])
            .push(creep);
    }

    harvester_allocator::allocate_creeps(role_map.remove(&creeps::Role::Harvester).unwrap());
    worker_allocator::allocate_creeps(role_map.remove(&creeps::Role::Worker).unwrap());
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
