use std::io::prelude::*;
use std::io::{Error, ErrorKind};
use std::path::Path;
use std::fs::File;
use std::net::{TcpListener, TcpStream};
use std::str::from_utf8;
use std::thread;
use rustc_serialize::json::decode;

use rori_utils::data::RoriData;
pub mod module_manager;
use core::module_manager::ModuleManager;

// TODO sslstream

struct Client {
    stream: TcpStream,
}

impl Client {
    fn new(stream: TcpStream) -> Client {
        return Client { stream: stream };
    }

    fn read(&mut self) -> String {
        let mut result = String::from("");
        let mut buffer = [0u8; 512];
        loop {
            let usize = self.stream.read(&mut buffer).unwrap();
            if usize == 0 {
                break;
            }
            let msg = from_utf8(&buffer).unwrap();
            result.push_str(msg);
        }
        result
    }
}


#[derive(Clone, RustcDecodable, RustcEncodable, Default, PartialEq, Debug)]
struct ConfigServer {
    pub ip: Option<String>,
    pub port: Option<String>,
}

pub struct Server {
    address: String,
}

impl Server {
    fn parse_config(data: String) -> String {
        let params: ConfigServer = decode(&data[..])
            .map_err(|_| {
                Error::new(ErrorKind::InvalidInput,
                           "Failed to decode configuration file.")
            })
            .unwrap();

        format!("{}:{}",
                &params.ip.unwrap_or(String::from("")),
                &params.port.unwrap_or(String::from("")))
    }

    pub fn new<P: AsRef<Path>>(config: P) -> Server {
        // Configure from file
        let mut file = File::open(config)
            .ok()
            .expect("Config file not found");
        let mut data = String::new();
        file.read_to_string(&mut data)
            .ok()
            .expect("failed to read!");
        let address = Server::parse_config(data);
        Server { address: address }
    }

    pub fn start(&mut self) {
        let listener = TcpListener::bind(&*self.address).unwrap();

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let client = Client::new(stream.try_clone().unwrap());
                    self.handle_client(client);
                }
                Err(e) => {
                    println!("Connection failed because {}", e);
                }
            }
        }

        drop(listener);
    }

    fn handle_client(&mut self, mut client: Client) {
        thread::spawn(move || {
            let data_received = client.read();
            let end = data_received.find(0u8 as char);
            let (data_received, _) = data_received.split_at(end.unwrap_or(data_received.len()));
            println!("[RECEIVED]:\n{}", data_received);
            let data_to_process = RoriData::from_json(String::from(data_received));
            let module_manager = ModuleManager::new(data_to_process);
            module_manager.process();
        });
    }
}
