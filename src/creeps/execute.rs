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

    match creep.get_mode() {
        super::Mode::Input => execute_input_mode(creep),
        super::Mode::Output => execute_output_mode(creep),
        super::Mode::Idle => debug!("Execute idle mode."),
    }
}

fn execute_input_mode(creep: &Creep) {
    if creep.get_target::<Source>().is_some() {
        harvest(creep);
    } else if creep.get_target::<Structure>().is_some() {
        transfer_from(creep);
    } else {
        idle(creep);
    }
}

fn execute_output_mode(creep: &Creep) {
    if creep.get_target::<StructureController>().is_some() {
        upgrade_controller(creep);
    } else if creep.get_target::<ConstructionSite>().is_some() {
        build(creep);
    } else {
        transfer_to(creep);
    }
}

fn transfer_to(creep: &Creep) {
    if let Some(target_structure) = creep.get_target::<Structure>() {
        if let Some(target_transferable) = target_structure.as_transferable() {
            let return_code = creep
                .creep
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

fn upgrade_controller(creep: &Creep) {
    if let Some(target_controller) = creep.get_target::<StructureController>() {
        let return_code = creep.creep.upgrade_controller(&target_controller);
        if return_code != ReturnCode::Ok {
            debug!("Failed upgrade_controller: {:?}", return_code);
        }
    } else {
        error!("Transfer to target is not a structure.");
    }
}

fn transfer_from(creep: &Creep) {
    assert!(creep.has_target());

    let target_structure = creep.get_target::<Structure>().unwrap();
    let target_withdrawable = target_structure.as_withdrawable().unwrap();
    let return_code = creep
        .creep
        .withdraw_all(target_withdrawable, ResourceType::Energy);
    if return_code != ReturnCode::Ok {
        debug!("Failed transfer_from: {:?}", return_code);
    }
}

fn harvest(creep: &Creep) {
    assert!(creep.has_target());

    let source = creep.get_target::<Source>().unwrap();
    let return_code = creep.creep.harvest(&source);
    if return_code != ReturnCode::Ok {
        debug!("Failed harvest: {:?}", return_code);
    }
}

fn build(creep: &Creep) {
    if let Some(construction_site) = creep.get_target::<ConstructionSite>() {
        let return_code = creep.creep.build(&construction_site);
        if return_code != ReturnCode::Ok {
            debug!("Failed build: {:?}", return_code);
        }
    } else {
        error!("Target is not a construction site.");
    }
}

fn idle(_: &Creep) {
    debug!("Idle");
}

fn move_to_target(creep: &Creep) {
    if let Some(target_position) = creep.get_target_position() {
        if target_position != creep.creep.pos() {
            let return_code = creep.creep.move_to(&target_position);
            if return_code == ReturnCode::Tired {
                debug!("Waiting for fatigue");
            } else if return_code != ReturnCode::Ok {
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
