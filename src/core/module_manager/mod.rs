use cpython::Python;
use regex::Regex;
use rori_utils::data::RoriData;
use rustc_serialize::json::decode;
use rustc_serialize::json::Json;
use std::fs::File;
use std::io::prelude::*;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

#[derive(Clone, RustcDecodable, RustcEncodable, Default, PartialEq, Debug)]
pub struct Module {
    pub name: String,
    pub desc: String,
    pub img: String,
    pub enabled: bool,
    pub condition: String,
    pub path: String,
    pub priority: u64,
}

#[allow(dead_code)]
pub struct ModuleManager {
    data: RoriData,
}

/**
 * When RORI receives a command, it needs to process this message.
 * The module manager tries to call all interresting modules.
 */
impl ModuleManager {
    pub fn new(data: RoriData) -> ModuleManager {
        return ModuleManager { data: data };
    }

    #[allow(unused_assignments)]
    pub fn process(&self) {
        // open modules/self.data.datatype.json
        let path = format!("rori_modules/{}.json", self.data.datatype);
        let mut file = File::open(path)
            .ok()
            .expect("Can't find modules for this type");
        let mut modules = String::new();
        file.read_to_string(&mut modules)
            .ok()
            .expect("failed to read module list");
        // parse json
        let modules_list = Json::from_str(&*modules).unwrap();
        // foreach priority, launch enabled modules if condition ok
        let mut priority = 0;


        // TODO Arc
        let stop: AtomicBool = AtomicBool::new(false);
        let stop_arc: Arc<AtomicBool> = Arc::new(stop);
        let module_found: AtomicBool = AtomicBool::new(false);
        let module_found_arc: Arc<AtomicBool> = Arc::new(module_found);
        while !stop_arc.load(Ordering::Relaxed) {
            let mut children = vec![];
            module_found_arc.store(false, Ordering::Relaxed);
            for item in modules_list.as_array().unwrap() {
                // TODO new thread

                let stop_arc_cloned: Arc<AtomicBool> = stop_arc.clone();
                let module_found_arc_cloned: Arc<AtomicBool> = module_found_arc.clone();
                let data_cloned = self.data.clone();
                let item_cloned = item.clone();


                children.push(thread::spawn(move || {
                    let module: Module = decode(&*item_cloned.to_string()).unwrap();
                    if module.priority == priority {
                        module_found_arc_cloned.store(true, Ordering::Relaxed);
                        info!(target:"module_manager", "Module found: {}", module.name);
                        // Parse text module
                        if module.enabled && data_cloned.datatype == "text" {
                            let re = Regex::new(&*module.condition).unwrap();
                            if re.is_match(&*data_cloned.content.to_lowercase()) {
                                info!(target:"module_manager", "{} match! Launch module...", module.name);
                                let continue_processing =
                                    ModuleManager::exec_module(module.path, data_cloned.clone());
                                info!(target:"module_manager", "{}: continue_processing: {}", module.name, continue_processing);
                                if !continue_processing {
                                    stop_arc_cloned.store(true, Ordering::Relaxed);
                                    return;
                                }
                            } else {
                                info!(target:"module_manager", "{} condition don't match", module.name);
                            }
                        } else if !module.enabled {
                            warn!(target:"module_manager", "Unknown datatype: {}", data_cloned.datatype);
                        }
                    }
                }));
            }

            for child in children {
                // Wait for the thread to finish. Returns a result.
                let _ = child.join();
            }

            if !module_found_arc.load(Ordering::Relaxed) {
                break;
            }
            priority += 1;
        }
    }

    fn exec_module(module: String, roridata: RoriData) -> bool {
        let py = Python::acquire_gil();
        let py = py.python();
        let load_module = py.import("rori_modules.load_module").unwrap();
        let continue_processing: bool =
            load_module.call(py, "exec_module", (module, roridata.to_string()), None)
                .unwrap()
                .extract(py)
                .unwrap();
        continue_processing
    }
}
