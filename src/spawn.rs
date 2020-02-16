use super::creeps::harvester::{Harvester, HarvesterManager};
use super::creeps::worker::{Worker, WorkerManager};
use super::creeps::CreepManager;
use log::*;
use screeps::{prelude::*, Part, ResourceType, ReturnCode};

struct Spawn(screeps::StructureSpawn);

impl Spawn {
    fn game_loop(&mut self, creep_manager: &CreepManager) {
        debug!("running spawn {}", self.0.name());

        if self.0.is_spawning() {
            return;
        }

        if let Some((body, name_prefix)) = self.get_spawn_target(creep_manager) {
            let spawn_cost = body.iter().map(|p| p.cost()).sum();
            if self.0.energy() >= spawn_cost {
                self.spawn_creep(&body, name_prefix);
            }
        }
    }

    fn get_spawn_target(&self, creep_manager: &CreepManager) -> Option<(Vec<Part>, &'static str)> {
        let harvester_spawn_target =
            self.get_harvester_spawn_target(&creep_manager.harvester_manager);
        if harvester_spawn_target.is_some() {
            return harvester_spawn_target;
        }

        let worker_spawn_target = self.get_worker_spawn_target(&creep_manager.worker_manager);
        if worker_spawn_target.is_some() {
            return worker_spawn_target;
        }

        return None;
    }

    fn get_harvester_spawn_target(
        &self,
        harvester_manager: &HarvesterManager,
    ) -> Option<(Vec<Part>, &'static str)> {
        if harvester_manager.get_target_source().is_some() {
            return Some(Harvester::get_description(
                self.0.store_capacity(Some(ResourceType::Energy)),
            ));
        }
        return None;
    }

    fn get_worker_spawn_target(
        &self,
        worker_manager: &WorkerManager,
    ) -> Option<(Vec<Part>, &'static str)> {
        if worker_manager.workers.len() > 0 {
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

    pub fn game_loop(&mut self, creep_manager: &CreepManager) {
        for spawn in &mut self.spawns {
            spawn.game_loop(creep_manager);
        }
    }
}
