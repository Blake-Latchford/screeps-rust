use log::*;
use screeps::{
    prelude::*, ConstructionSite, ResourceType, ReturnCode, Source, Structure, StructureController,
};

use super::Creep;

pub fn execute(creep: &Creep) {
    debug!("running {}", creep.creep.name());
    if creep.creep.spawning() {
        return;
    }

    update_mode(creep);
    execute_mode(creep);
    update_mode(creep);
    move_to_target(creep);
}

fn update_mode(creep: &Creep) {
    if let Some(mode) = creep.get_new_mode() {
        creep.set_mode(mode);
    }
}

fn execute_mode(creep: &Creep) {
    if !creep.has_target() {
        warn!("No target selected!");
        return;
    }

    let return_code = match creep.get_mode() {
        super::Mode::Input => execute_input_mode(creep),
        super::Mode::Output => execute_output_mode(creep),
        super::Mode::Idle => ReturnCode::Ok,
    };
    if return_code == ReturnCode::NotInRange {
        debug!(
            "Failed '{:?}' to '{:?}': {:?}",
            creep.get_mode(),
            creep.get_target_id(),
            return_code
        );
    } else if return_code != ReturnCode::Ok {
        error!(
            "Failed '{:?}' to '{:?}': {:?}",
            creep.get_mode(),
            creep.get_target_id(),
            return_code
        );
    }
}

fn execute_input_mode(creep: &Creep) -> ReturnCode {
    if creep.get_target::<Source>().is_some() {
        debug!("harvest");
        return harvest(creep);
    }
    debug!("transfer_from");
    return transfer_from(creep);
}

fn execute_output_mode(creep: &Creep) -> ReturnCode {
    if creep.get_target::<StructureController>().is_some() {
        debug!("upgrade_controller");
        return upgrade_controller(creep);
    }
    if creep.get_target::<ConstructionSite>().is_some() {
        debug!("build");
        return build(creep);
    }
    debug!("transfer_to");
    return transfer_to(creep);
}

fn transfer_to(creep: &Creep) -> ReturnCode {
    if let Some(target_structure) = creep.get_target::<Structure>() {
        if let Some(target_transferable) = target_structure.as_transferable() {
            return creep
                .creep
                .transfer_all(target_transferable, ResourceType::Energy);
        }
    }
    return ReturnCode::InvalidTarget;
}

fn upgrade_controller(creep: &Creep) -> ReturnCode {
    if let Some(target_controller) = creep.get_target::<StructureController>() {
        return creep.creep.upgrade_controller(&target_controller);
    }
    return ReturnCode::InvalidTarget;
}

fn transfer_from(creep: &Creep) -> ReturnCode {
    assert!(creep.has_target());

    if let Some(target_structure) = creep.get_target::<Structure>() {
        if let Some(target_withdrawable) = target_structure.as_withdrawable() {
            return creep
                .creep
                .withdraw_all(target_withdrawable, ResourceType::Energy);
        }
    }
    return ReturnCode::InvalidTarget;
}

fn harvest(creep: &Creep) -> ReturnCode {
    assert!(creep.has_target());

    if let Some(source) = creep.get_target::<Source>() {
        return creep.creep.harvest(&source);
    }
    return ReturnCode::InvalidTarget;
}

fn build(creep: &Creep) -> ReturnCode {
    if let Some(construction_site) = creep.get_target::<ConstructionSite>() {
        return creep.creep.build(&construction_site);
    }
    return ReturnCode::InvalidTarget;
}

fn move_to_target(creep: &Creep) {
    if let Some(target_position) = creep.get_target_position() {
        if target_position != creep.creep.pos() {
            let return_code = creep.creep.move_to(&target_position);
            if return_code != ReturnCode::Ok {
                debug!("Failed move: {:?}", return_code);
            }
        } else {
            move_random_direction(creep);
        }
    } else {
        debug!("No move target");
    }
}

fn move_random_direction(creep: &Creep) {
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
        let return_code = creep.creep.move_direction(*direction);
        if return_code == ReturnCode::Ok {
            break;
        } else {
            debug!("Failed move {:?}: {:?}", direction, return_code);
        }
    }
}
