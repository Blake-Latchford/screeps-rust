use log::*;
use screeps::objects::HasPosition;
use screeps::{prelude::*, Position, RawObjectId};
use std::collections::HashSet;
use std::str::FromStr;

mod execute;

#[derive(PartialEq, Debug)]
pub enum Mode {
    Input,
    Output,
    Idle,
}

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub enum Role {
    Harvester,
    Worker,
}

const ROLE_STRINGS: [(Role, &'static str); 2] =
    [(Role::Harvester, "harvester"), (Role::Worker, "worker")];

impl FromStr for Role {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        for (role, role_string) in ROLE_STRINGS.iter() {
            if s == *role_string {
                return Ok(role.clone());
            }
        }
        return Err("Invalid mode string:".to_owned() + &s.to_owned());
    }
}

impl Role {
    pub fn to_string(&self) -> &'static str {
        for (role, role_string) in ROLE_STRINGS.iter() {
            if self == role {
                return role_string;
            }
        }

        panic!("Unable to convert role to string");
    }
}

pub struct Creep {
    creep: screeps::Creep,
    pub role: Role,
}

impl Creep {
    pub fn new(creep: screeps::Creep) -> Creep {
        let name = creep.name();
        let name_prefix = name.split(":").next().unwrap();
        let role = Role::from_str(name_prefix).unwrap();

        Creep {
            creep: creep,
            role: role,
        }
    }
    fn get_mode(&self) -> Mode {
        return match self.get_mode_string().as_str() {
            "input" => Mode::Input,
            "output" => Mode::Output,
            _ => Mode::Idle,
        };
    }

    fn get_mode_string(&self) -> String {
        if let Ok(Some(result)) = self.creep.memory().string("mode") {
            return result;
        }
        return "".to_string();
    }

    fn set_mode(&self, mode: Mode) {
        if self.get_mode() == mode {
            return;
        }

        let mode_string = match mode {
            Mode::Input => "input",
            Mode::Output => "output",
            Mode::Idle => "idle",
        };
        self.creep.memory().set("mode", mode_string);
        self.creep.say(mode_string, false);
        debug!("{}: {}", self.creep.name(), mode_string);
    }

    fn has_target(&self) -> bool {
        if let Some(stored_id) = self.get_stored_id(self.get_target_key()) {
            return screeps::game::get_object_erased(stored_id).is_some();
        }
        return false;
    }

    fn get_target_position(&self) -> Option<Position> {
        let target_id = self.get_stored_id(self.get_target_key())?;
        Some(screeps::game::get_object_erased(target_id)?.pos())
    }

    fn get_target<T>(&self) -> Option<T>
    where
        T: screeps::SizedRoomObject + screeps::HasId,
    {
        return self.get_stored_object(self.get_target_key());
    }

    fn get_target_key(&self) -> &'static str {
        return match self.get_mode() {
            Mode::Input => "input",
            Mode::Output => "output",
            Mode::Idle => "input",
        };
    }

    pub fn get_input<T>(&self) -> Option<T>
    where
        T: screeps::SizedRoomObject + screeps::HasId,
    {
        return self.get_stored_object("input");
    }

    pub fn set_input(&self, target_id: RawObjectId) {
        debug!(
            "{}:{}: set input target {}",
            std::file!(),
            std::line!(),
            target_id.to_string()
        );
        self.creep.memory().set("input", target_id.to_string());
    }

    pub fn get_output<T>(&self) -> Option<T>
    where
        T: screeps::SizedRoomObject + screeps::HasId,
    {
        return self.get_stored_object("output");
    }

    pub fn set_output(&self, target_id: RawObjectId) {
        debug!(
            "{}:{}: set out target {}",
            std::file!(),
            std::line!(),
            target_id.to_string()
        );
        self.creep.memory().set("output", target_id.to_string());
    }

    fn get_stored_object<T>(&self, key: &str) -> Option<T>
    where
        T: screeps::SizedRoomObject + screeps::HasId,
    {
        let stored_id = self.get_stored_id(key)?;
        screeps::game::get_object_typed::<T>(stored_id.into()).ok()?
    }

    pub fn get_stored_id(&self, key: &str) -> Option<RawObjectId> {
        let stored_target_id_string = self.creep.memory().string(key).ok()??;
        let id = stored_target_id_string.parse().unwrap();
        return Some(id);
    }

    fn get_new_mode(&self) -> Option<Mode> {
        if self.is_full() {
            return Some(Mode::Output);
        } else if self.is_empty() {
            return Some(Mode::Input);
        }

        return None;
    }

    fn is_full(&self) -> bool {
        self.creep.store_free_capacity(None) == 0
    }

    fn is_empty(&self) -> bool {
        self.creep.store_used_capacity(None) == 0
    }

    pub fn get_range_to<T>(&self, target: &T) -> u32
    where
        T: ?Sized + HasPosition,
    {
        self.creep.pos().get_range_to(target)
    }
}

pub fn game_loop() {
    for screeps_creep in screeps::game::creeps::values() {
        let creep = Creep::new(screeps_creep);
        execute::execute(&creep);
    }
    cleanup_memory().expect("expected Memory.creeps format to be a regular memory object");
}

fn cleanup_memory() -> Result<(), Box<dyn std::error::Error>> {
    let time = screeps::game::time();
    if time % 32 != 3 {
        return Ok(());
    }

    info!("running memory cleanup");

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
