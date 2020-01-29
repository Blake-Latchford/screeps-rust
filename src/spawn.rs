use log::*;
use screeps::{find, prelude::*, ObjectId, Part, ResourceType, ReturnCode};

pub fn game_loop() {
    for spawn in screeps::game::spawns::values() {
        Spawn(spawn).game_loop();
    }
}

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
        self.update_next_target_source();
        if get_target(screeps::memory::root()).is_some() {
            return Some(super::creeps::harvester::get_description(
                self.0.store_capacity(Some(ResourceType::Energy)),
            ));
        }
        return None;
    }

    fn update_next_target_source(&self) {
        if get_target(screeps::memory::root()).is_some() {
            return;
        }

        let mut sources = self.0.room().find(find::SOURCES);
        for creep in screeps::game::creeps::values() {
            if let Some(creep_target) = get_target(creep.memory()) {
                if let Some(index) = sources.iter().position(|x| *x == creep_target) {
                    sources.remove(index);
                }
            }
        }
        sources.sort_by_key(|s| s.pos().get_range_to(&self.0.pos()));
        if sources.len() > 0 {
            screeps::memory::root().set("target", sources[0].id().to_string());
        }
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

fn get_target(mem: screeps::memory::MemoryReference) -> Option<screeps::Source> {
    let target_string = mem.string("target").ok()??;
    let target_id: ObjectId<screeps::Source> = target_string.parse().ok()?;
    screeps::game::get_object_typed(target_id).ok()?
}
