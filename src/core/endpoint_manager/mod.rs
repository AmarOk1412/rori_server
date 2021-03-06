use openssl::ssl::{SslContext, SslMethod, Ssl};
use rori_utils::data::RoriData;
use std::net::TcpStream;
use std::io::Write;

#[derive(Clone, PartialEq, Eq, RustcEncodable)]
pub struct Endpoint {
    pub name: String,
    pub id: u64,
    pub owner: String,
    pub address: String,
    pub compatible_data: Vec<String>,
}

#[derive(Clone)]
pub struct EndpointManager {
    endpoints: Vec<Endpoint>,
    id: u64,
}

/**
 * Manage endpoints
 */
impl EndpointManager {
    pub fn new() -> EndpointManager {
        EndpointManager {
            endpoints: Vec::new(),
            id: 0,
        }
    }

    /**
     * Save an endpoint if it's not a duplicate
     */
    pub fn register_endpoint(&mut self, data: RoriData) {
        // if we receive a register command
        if data.datatype == "register" {
            // content = ip:port|type1|type2...
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
                id: self.id,
                owner: data.author,
                address: address,
                compatible_data: content_part,
            };
            // Avoid duplicates
            if !self.endpoint_already_exists(&endpoint) {
                self.endpoints.push(endpoint);
                self.id += 1;
            }
        }
    }

    /**
     * Get if an endpoint already exists
     * @return if we find a endpoint
     */
    pub fn endpoint_already_exists(&mut self, endpoint: &Endpoint) -> bool {
        for elem in self.endpoints.clone() {
            if endpoint.name == elem.name && endpoint.owner == elem.owner &&
               elem.address == endpoint.address &&
               elem.compatible_data == endpoint.compatible_data {
                return true;
            }
        }
        return false;
    }

    /**
     * Remove an endpoint if exists
     * @return if the endpoint is removed
     */
    pub fn remove_endpoint(&mut self, id_to_rm: u64) -> bool {
        let index_to_remove = self.get_endpoint_index(id_to_rm);
        if index_to_remove >= 0 {
            self.endpoints.remove(index_to_remove as usize);
            return true;
        }
        return false;
    }

    /**
     * Find an endpoint by id
     * @return the position of the endpoint
     */
    fn get_endpoint_index(&mut self, id_to_rm: u64) -> i64 {
        let mut cpt = 0;
        for endpoint in self.endpoints.clone() {
            if id_to_rm == endpoint.id {
                return cpt;
            }
            cpt += 1;
        }
        -1
    }

    /**
     * Get endpoints
     */
    pub fn get_endpoint_for_type(&self, datatype: String, owner: String) -> Vec<Endpoint> {
        let mut result: Vec<Endpoint> = Vec::new();
        let endpoints = self.endpoints.clone();
        for endpoint in endpoints {
            if endpoint.compatible_data.contains(&datatype) {
                if owner.len() == 0 || owner == endpoint.owner || endpoint.owner == "*" {
                    result.push(endpoint);
                }
            }
        }
        result
    }

    /**
     * send a data to the endpoint with the given id
     */
    pub fn send_to_endpoint(&self, id: u64, data: &String) {
        let endpoints = self.endpoints.clone();
        for endpoint in endpoints {
            if endpoint.id == id {
                let context = SslContext::builder(SslMethod::tls()).unwrap();
                let ssl = context.build();
                let inner = TcpStream::connect(&*endpoint.address).unwrap();
                if let Ok(mut stream) = Ssl::new(&ssl).unwrap().connect(inner) {
                    let _ = stream.write(data.to_string().as_bytes());
                } else {
                    error!(target:"RoriClient", "Couldn't connect to RORI at address {}", &*endpoint.address);
                }
            }
        }
    }
}
