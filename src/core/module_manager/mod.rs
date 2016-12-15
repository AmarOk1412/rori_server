
use rori_utils::data::RoriData;
use rustc_serialize::json::decode;
use rustc_serialize::json::Json;
use std::fs::File;
use std::io::prelude::*;
use regex::Regex;

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

impl ModuleManager {
    pub fn new(data: RoriData) -> ModuleManager {
        return ModuleManager { data: data };
    }

    pub fn process(&self) {
        // open modules/self.data.datatype.json
        let path = format!("modules/{}.json", self.data.datatype);
        let mut file = File::open(path)
            .ok()
            .expect("Can't find modules for this type");
        let mut modules = String::new();
        file.read_to_string(&mut modules)
            .ok()
            .expect("failed to read module list");
        // parse json
        let modules_list = Json::from_str(&*modules).unwrap();
        let mut stop = false;
        let mut priority = 0;
        let mut module_found = false;
        while !stop {
            module_found = false;
            for item in modules_list.as_array() {
                let module: Module = decode(&*item[0].to_string()).unwrap();
                if module.priority == priority {
                    module_found = true;
                    println!("Module found: {}", module.name);
                    let re = Regex::new(&*module.condition).unwrap();
                    if module.enabled && re.is_match(&*self.data.content) {
                        println!("The module match!");
                        stop = true;
                    }
                }
            }
            if !module_found {
                break;
            }
            priority += 1;
        }
        // foreach priority, launch enabled modules if condition ok
        if self.data.datatype == "text" {
            println!("Process plain text data");
        } else {
            println!("Print unknown datatype: {}", self.data.datatype);
        }
    }
}
