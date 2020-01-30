use log::*;
use std::collections::HashSet;

pub mod harvester;

trait Creep {
    fn new(creep: screeps::Creep) -> Self;
    fn game_loop(self);
}

pub fn game_loop() {
    debug!("running creeps");
    for creep in screeps::game::creeps::values() {
        if creep.name().starts_with(harvester::NAME_PREFIX) {
            harvester::Harvester::new(creep).game_loop();
        }
    }

    let time = screeps::game::time();
    if time % 32 == 3 {
        info!("running memory cleanup");
        cleanup_memory().expect("expected Memory.creeps format to be a regular memory object");
    }

    info!("done! cpu: {}", screeps::game::cpu::get_used())
}

fn cleanup_memory() -> Result<(), Box<dyn std::error::Error>> {
    let alive_creeps: HashSet<String> = screeps::game::creeps::keys().into_iter().collect();

    let screeps_memory = match screeps::memory::root().dict("creeps")? {
        Some(v) => v,
        None => {
            warn!("not cleaning game creep memory: no Memory.creeps dict");
            return Ok(());
        }
    };

    for mem_name in screeps_memory.keys() {
        if !alive_creeps.contains(&mem_name) {
            debug!("cleaning up creep memory of dead creep {}", mem_name);
            screeps_memory.del(&mem_name);
        }
    }

    Ok(())
}
