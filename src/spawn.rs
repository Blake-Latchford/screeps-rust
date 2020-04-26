use super::creeps::harvester::Harvester;
use super::creeps::worker::Worker;
use log::*;
use screeps::{prelude::*, Part, ResourceType, ReturnCode};

struct Spawn(screeps::StructureSpawn);

impl Spawn {
    fn game_loop(&mut self) {
        debug!("running spawn {}", self.0.name());

        if self.0.is_spawning() {
            return;
        }

        if let Some((body, name_prefix)) = self.get_spawn_target() {
            let spawn_cost = body.iter().map(|p| p.cost()).sum();
            if self.0.energy() >= spawn_cost {
                self.spawn_creep(&body, name_prefix);
            }
        }
    }

    fn get_spawn_target(&self) -> Option<(Vec<Part>, &'static str)> {
        let harvester_spawn_target = self.get_harvester_spawn_target();
        if harvester_spawn_target.is_some() {
            return harvester_spawn_target;
        }

        return None;
    }

    fn get_harvester_spawn_target(&self) -> Option<(Vec<Part>, &'static str)> {
        debug!("Check for harvester targets.");
        if Harvester::get_target_source().is_some() {
            let store_capacity = self.0.store_capacity(Some(ResourceType::Energy));
            if store_capacity > 0 {
                return Some(Harvester::get_description(store_capacity));
            } else {
                error!("Store has no capacity!");
            }
        } else {
            debug!("No harvester targets");
        }
        return None;
    }

    fn get_worker_spawn_target(&self, workers: &Vec<Worker>) -> Option<(Vec<Part>, &'static str)> {
        if workers.len() > 0 {
            return None;
        }
        return Some(Worker::get_description(
            self.0.store_capacity(Some(ResourceType::Energy)),
        ));
    }

    fn spawn_creep(&mut self, body: &Vec<screeps::Part>, name_prefix: &str) {
        for i in 0..1000 {
            let name = name_prefix.to_owned() + &i.to_string();
            let return_code = self.0.spawn_creep(&body, &name);
            match return_code {
                ReturnCode::NameExists => continue,
                ReturnCode::Ok => return,
                _ => warn!("couldn't spawn: {:?}", return_code),
            }
            return;
        }
    }
}

pub struct SpawnManager {
    spawns: Vec<Spawn>,
}

impl SpawnManager {
    pub fn new() -> SpawnManager {
        let mut spawn_manager = SpawnManager { spawns: Vec::new() };
        spawn_manager.register_all();
        return spawn_manager;
    }

    fn register_all(&mut self) {
        for spawn in screeps::game::spawns::values() {
            self.spawns.push(Spawn(spawn));
        }
    }

    pub fn game_loop(&mut self) {
        for spawn in &mut self.spawns {
            spawn.game_loop();
        }
    }
}
