
use rori_utils::data::RoriData;

#[allow(dead_code)]
pub struct ModuleManager {
    data: RoriData,
}

impl ModuleManager {
    pub fn new(data: RoriData) -> ModuleManager {
        return ModuleManager { data: data };
    }

    pub fn process(&self) {
        if self.data.datatype == "text" {
            println!("Process plain text data");
        } else {
            println!("Print unknown datatype: {}", self.data.datatype);
        }
    }
}
