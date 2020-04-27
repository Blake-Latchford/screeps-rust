use log::*;

pub mod harvester_allocator;
pub mod worker_allocator;

pub fn allocate_creeps() {
    for creep in screeps::game::creeps::values() {
        allocate_creep(creep);
    }
}

fn allocate_creep(creep: screeps::Creep) {
    let name = creep.name();
    let name_prefix = name.split(":").next().unwrap();
    match name_prefix {
        harvester_allocator::NAME_PREFIX => harvester_allocator::allocate_creep(creep),
        worker_allocator::NAME_PREFIX => worker_allocator::allocate_creep(creep),
        _ => error!("Invalid name prefix: {}", name_prefix),
    }
}
