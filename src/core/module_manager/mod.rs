use cpython::Python;
use regex::Regex;
use rori_utils::data::RoriData;
use rustc_serialize::json::decode;
use rustc_serialize::json::Json;
use std::fs::File;
use std::io::prelude::*;

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
        let mut stop = false;
        let mut module_found = false;
        while !stop {
            module_found = false;
            for item in modules_list.as_array().unwrap() {
                // TODO new thread
                let module: Module = decode(&*item.to_string()).unwrap();
                if module.priority == priority {
                    module_found = true;
                    info!(target:"module_manager", "Module found: {}", module.name);
                    // Parse text module
                    if module.enabled && self.data.datatype == "text" {
                        let re = Regex::new(&*module.condition).unwrap();
                        if re.is_match(&*self.data.content.to_lowercase()) {
                            info!(target:"module_manager", "The module match! Launch module...");
                            let continue_processing = ModuleManager::exec_module(module.path,
                                                                                 self.data.clone());
                            info!(target:"module_manager", "continue_processing: {}", continue_processing);
                            if !continue_processing {
                                stop = true;
                                info!(target:"module_manager", "Stop processing modules");
                                break;
                            }
                        } else {
                            info!(target:"module_manager", "condition don't match");
                        }
                    } else if !module.enabled {
                        warn!(target:"module_manager", "Unknown datatype: {}", self.data.datatype);
                    }

                }
            }
            if !module_found {
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
