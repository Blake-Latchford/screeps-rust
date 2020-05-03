use crate::allocator;
use log::*;
use screeps::{prelude::*, ResourceType, ReturnCode};

struct Spawn(screeps::StructureSpawn);

impl Spawn {
    fn game_loop(&mut self) {
        debug!("running spawn {}", self.0.name());

        if self.0.is_spawning() {
            return;
        }

        if let Some((body, role)) = allocator::get_spawn_target(self.capacity()) {
            self.spawn_creep(&body, role.to_string());
        }
    }

    fn spawn_creep(&mut self, body: &Vec<screeps::Part>, name_prefix: &str) {
        for i in 0..1000 {
            let name = name_prefix.to_owned() + ":" + &i.to_string();
            let return_code = self.0.spawn_creep(&body, &name);
            match return_code {
                ReturnCode::NameExists => continue,
                ReturnCode::Ok => return,
                _ => warn!("couldn't spawn: {:?}", return_code),
            }
            return;
        }
    }

    fn capacity(&self) -> u32 {
        self.0.store_capacity(Some(ResourceType::Energy))
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
