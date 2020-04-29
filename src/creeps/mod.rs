use log::*;
use screeps::objects::HasPosition;
use screeps::{
    prelude::*, ConstructionSite, Position, RawObjectId, ResourceType, ReturnCode, Source,
    Structure, StructureController,
};
use std::collections::HashSet;

pub mod harvester;
pub mod worker;

#[derive(PartialEq, Debug)]
pub enum Mode {
    Harvest,
    TransferTo,
    TransferFrom,
    UpgradeController,
    Build,
    Idle,
}

impl Mode {
    fn is_input_mode(&self) -> bool {
        const INPUT_MODES: [Mode; 3] = [Mode::Harvest, Mode::TransferFrom, Mode::Idle];
        INPUT_MODES.contains(self)
    }
}

pub trait Creep {
    fn get_creep(&self) -> &screeps::Creep;
    fn get_new_mode(&self) -> Option<Mode>;
    fn get_new_target(&self) -> Option<RawObjectId>;

    fn game_loop(&self) {
        debug!("running {}", self.get_creep().name());
        if self.get_creep().spawning() {
            return;
        }

        self.update_mode();
        self.execute_mode();
        self.update_mode();
        self.move_to_target();
    }

    fn update_mode(&self) {
        if let Some(mode) = self.get_new_mode() {
            self.set_mode(mode);
        }
    }

    fn execute_mode(&self) {
        if !self.has_target() {
            warn!("No target selected!");
            return;
        }

        let mode = self.get_mode();
        debug!("Execute mode {:?}", mode);
        match mode {
            Mode::TransferTo => self.transfer_to(),
            Mode::TransferFrom => self.transfer_from(),
            Mode::Harvest => self.harvest(),
            Mode::UpgradeController => self.upgrade_controller(),
            Mode::Build => self.build(),
            Mode::Idle => self.idle(),
        }
    }

    fn transfer_to(&self) {
        if let Some(target_structure) = self.get_target::<Structure>() {
            if let Some(target_transferable) = target_structure.as_transferable() {
                let return_code = self
                    .get_creep()
                    .transfer_all(target_transferable, ResourceType::Energy);
                if return_code == ReturnCode::NotInRange {
                    debug!("Failed transfer_to: {:?}", return_code);
                } else if return_code != ReturnCode::Ok {
                    error!(
                        "Failed transfer_to '{:?}': {:?}",
                        target_structure.id(),
                        return_code
                    );
                }
            } else {
                error!("Transfer to target is not transferable or upgradable");
            }
        } else {
            error!("Transfer to target is not a structure.");
        }
    }

    fn upgrade_controller(&self) {
        if let Some(target_controller) = self.get_target::<StructureController>() {
            let return_code = self.get_creep().upgrade_controller(&target_controller);
            if return_code != ReturnCode::Ok {
                debug!("Failed upgrade_controller: {:?}", return_code);
            }
        } else {
            error!("Transfer to target is not a structure.");
        }
    }

    fn transfer_from(&self) {
        assert!(self.has_target());

        let target_structure = self.get_target::<Structure>().unwrap();
        let target_withdrawable = target_structure.as_withdrawable().unwrap();
        let return_code = self
            .get_creep()
            .withdraw_all(target_withdrawable, ResourceType::Energy);
        if return_code != ReturnCode::Ok {
            debug!("Failed transfer_from: {:?}", return_code);
        }
    }

    fn harvest(&self) {
        assert!(self.has_target());

        let source = self.get_target::<Source>().unwrap();
        let return_code = self.get_creep().harvest(&source);
        if return_code != ReturnCode::Ok {
            debug!("Failed harvest: {:?}", return_code);
        }
    }

    fn build(&self) {
        if let Some(construction_site) = self.get_target::<ConstructionSite>() {
            let return_code = self.get_creep().build(&construction_site);
            if return_code != ReturnCode::Ok {
                debug!("Failed build: {:?}", return_code);
            }
        } else {
            error!("Target is not a construction site.");
        }
    }

    fn idle(&self) {
        debug!("Idle");
    }

    fn move_to_target(&self) {
        if let Some(target_position) = self.get_target_position() {
            if target_position != self.get_creep().pos() {
                let return_code = self.get_creep().move_to(&target_position);
                if return_code == ReturnCode::Tired {
                    debug!("Waiting for fatigue");
                } else if return_code != ReturnCode::Ok {
                    debug!("Failed move: {:?}", return_code);
                }
            } else {
                self.move_random_direction();
            }
        } else {
            debug!("No move target");
        }
    }

    fn move_random_direction(&self) {
        let directions = [
            screeps::Direction::Top,
            screeps::Direction::TopRight,
            screeps::Direction::Right,
            screeps::Direction::BottomRight,
            screeps::Direction::Bottom,
            screeps::Direction::BottomLeft,
            screeps::Direction::Left,
            screeps::Direction::TopLeft,
        ];
        for direction in directions.iter() {
            let return_code = self.get_creep().move_direction(*direction);
            if return_code == ReturnCode::Ok {
                break;
            } else {
                debug!("Failed move {:?}: {:?}", direction, return_code);
            }
        }
    }

    fn get_mode(&self) -> Mode {
        return match self.get_mode_string().as_str() {
            "h" => Mode::Harvest,
            "tt" => Mode::TransferTo,
            "tf" => Mode::TransferFrom,
            "u" => Mode::UpgradeController,
            "b" => Mode::Build,
            _ => Mode::Idle,
        };
    }

    fn get_mode_string(&self) -> String {
        if let Ok(Some(result)) = self.get_creep().memory().string("mode") {
            return result;
        }
        return "".to_string();
    }

    fn set_mode(&self, mode: Mode) {
        if self.get_mode() == mode {
            return;
        }

        let mode_string = match mode {
            Mode::Harvest => "h",
            Mode::TransferTo => "tt",
            Mode::TransferFrom => "tf",
            Mode::UpgradeController => "u",
            Mode::Build => "b",
            Mode::Idle => "i",
        };
        self.get_creep().memory().set("mode", mode_string);
        let return_code = self.get_creep().say(mode_string, false);
        if return_code != ReturnCode::Ok {
            debug!("say: {:?}", return_code);
        }
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
        return match self.get_mode().is_input_mode() {
            true => "input",
            false => "output",
        };
    }

    fn get_stored_object<T>(&self, key: &str) -> Option<T>
    where
        T: screeps::SizedRoomObject + screeps::HasId,
    {
        let stored_id = self.get_stored_id(key)?;
        screeps::game::get_object_typed::<T>(stored_id.into()).ok()?
    }

    fn get_stored_id(&self, key: &str) -> Option<RawObjectId> {
        let stored_target_id_string = self.get_creep().memory().string(key).ok()??;
        let id = stored_target_id_string.parse().unwrap();
        return Some(id);
    }
    fn is_mode(&self, mode: Mode) -> bool {
        self.get_mode() == mode
    }

    fn has_capacity(&self) -> bool {
        self.get_creep().store_free_capacity(None) != 0
    }

    fn is_full(&self) -> bool {
        self.get_creep().store_free_capacity(None) == 0
    }

    fn is_empty(&self) -> bool {
        self.get_creep().store_used_capacity(None) == 0
    }
}

pub struct CreepManager {
    pub workers: Vec<worker::Worker>,
    pub harvesters: Vec<harvester::Harvester>,
}

impl CreepManager {
    pub fn new() -> CreepManager {
        debug!("register creeps");
        let mut creep_manager = CreepManager {
            workers: vec![],
            harvesters: vec![],
        };
        creep_manager.register_all_creeps();
        return creep_manager;
    }

    fn register_all_creeps(&mut self) {
        for creep in screeps::game::creeps::values() {
            if creep.name().starts_with(harvester::NAME_PREFIX) {
                self.harvesters.push(harvester::Harvester(creep))
            } else if creep.name().starts_with(worker::NAME_PREFIX) {
                self.workers.push(worker::Worker(creep));
            }
        }
    }

    pub fn game_loop(&self) {
        debug!("running creeps");

        for harvester in &self.harvesters {
            harvester.game_loop();
        }
        for worker in &self.workers {
            worker.game_loop();
        }
        CreepManager::cleanup_memory()
            .expect("expected Memory.creeps format to be a regular memory object");
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
}
