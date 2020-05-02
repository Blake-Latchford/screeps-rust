use super::creeps;

pub mod harvester_allocator;
pub mod worker_allocator;

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
