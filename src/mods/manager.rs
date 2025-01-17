use crate::config::{MODS_ENABLED, MODS_PATH};
use crate::mods::vm::Machine;

pub fn initialize_mods() -> std::io::Result<()> {
    if !MODS_ENABLED {
        info!("Mods are disabled, skipping initialization");
        Ok(())
    } else {
        let mods_list = match std::fs::read_to_string(MODS_PATH) {
            Ok(s) => s,
            Err(e) => {return Err(e)}
        };
        
        let mut failed_mods = 0;
        
        info!("Initializing mods");
        for init_mod in mods_list.lines() {
            info!("Initializing mod {}", init_mod);
            error!("Mod {} not found, skipping", init_mod);
            
            failed_mods += 1;
        }
        
        if failed_mods > 0 {
            warn!("{} mods failed, {} mods started", failed_mods, 0);
        }
        
        Ok(())
    }
}

pub fn start_mods() {
    let machine = Machine::new("");
    let mut machine = machine.execute_threaded().join().unwrap();
    info!("Output from mod virtual machine: {:?}", machine.read_str_from_vm());
}
