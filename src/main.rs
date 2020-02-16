use log::*;
use stdweb::js;

mod creeps;
mod logging;
mod spawn;

fn main() {
    logging::setup_logging(logging::Info);

    js! {
        var game_loop = @{game_loop};

        module.exports.loop = function() {
            // Provide actual error traces.
            try {
                game_loop();
            } catch (error) {
                // console_error function provided by 'screeps-game-api'
                console_error("caught exception:", error);
                if (error.stack) {
                    console_error("stack trace:", error.stack);
                }
                console_error("resetting VM next tick.");
                // reset the VM since we don't know if everything was cleaned up and don't
                // want an inconsistent state.
                module.exports.loop = wasm_initialize;
            }
        }
    }
}

fn game_loop() {
    debug!("loop starting! CPU: {}", screeps::game::cpu::get_used());
    let creep_manager = creeps::CreepManager::new();
    let mut spawn_manager = spawn::SpawnManager::new();

    spawn_manager.game_loop(&creep_manager);
    creep_manager.game_loop();
    info!("done! cpu: {}", screeps::game::cpu::get_used())
}
