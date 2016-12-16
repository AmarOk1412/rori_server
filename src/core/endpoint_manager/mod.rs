use rori_utils::data::RoriData;

#[derive(Clone)]
pub struct Endpoint {
    pub name: String,
    pub owner: String,
    pub address: String,
    pub compatible_data: Vec<String>,
}

#[derive(Clone)]
pub struct EndpointManager {
    endpoints: Vec<Endpoint>,
}

impl EndpointManager {
    pub fn new() -> EndpointManager {
        EndpointManager { endpoints: Vec::new() }
    }

    pub fn register_endpoint(&mut self, data: RoriData) {
        if data.datatype == "register" {
            let mut collected_part: Vec<&str> = data.content.split('|').collect();
            let mut address = String::from("");
            let mut content_part: Vec<String> = Vec::new();
            while collected_part.len() > 0 {
                let collected = String::from(collected_part.pop().unwrap_or(""));
                if collected_part.len() == 0 {
                    address = collected;
                } else {
                    content_part.push(collected);
                }
            }
            let endpoint = Endpoint {
                name: data.client,
                owner: data.author,
                address: address,
                compatible_data: content_part,
            };
            self.endpoints.push(endpoint);
        }
    }

    pub fn get_endpoint_for_type(&self, datatype: String) -> Vec<Endpoint> {
        let mut result: Vec<Endpoint> = Vec::new();
        let endpoints = self.endpoints.clone();
        for endpoint in endpoints {
            if endpoint.compatible_data.contains(&datatype) {
                result.push(endpoint);
            }
        }
        result
    }
}
