
pub mod endpoint_manager;
pub mod module_manager;

use core::module_manager::ModuleManager;
use core::endpoint_manager::EndpointManager;
use iron::prelude::*;
use iron::status;
use openssl::ssl::{SslContext, SslMethod, SslStream, SSL_VERIFY_NONE};
use openssl::x509::X509FileType::PEM;
use rori_utils::data::RoriData;
use router::Router;
use rustc_serialize::json::{self, decode};
use std::fs::File;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::str::from_utf8;
use std::sync::Mutex;
use std::thread;

lazy_static! {
    static ref ENDPOINTMANAGER: Mutex<EndpointManager> = Mutex::new(EndpointManager::new());
}

struct Client {
    stream: SslStream<TcpStream>,
}

impl Client {
    fn new(stream: SslStream<TcpStream>) -> Client {
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
    pub cert: Option<String>,
    pub key: Option<String>,
}

pub struct Server {
    address: String,
    cert: String,
    key: String,
}

impl Server {
    pub fn new<P: AsRef<Path>>(config: P) -> Server {
        // Configure from file
        let mut file = File::open(config)
            .ok()
            .expect("Config file not found");
        let mut data = String::new();
        file.read_to_string(&mut data)
            .ok()
            .expect("failed to read!");
        let params: ConfigServer = decode(&data[..]).unwrap();
        let address = format!("{}:{}",
                              &params.ip.unwrap_or(String::from("")),
                              &params.port.unwrap_or(String::from("")));

        Server {
            address: address,
            cert: params.cert.unwrap_or(String::from("")),
            key: params.key.unwrap_or(String::from("")),
        }
    }

    pub fn start(&mut self) {
        let listener = TcpListener::bind(&*self.address).unwrap();
        let mut ssl_context = SslContext::new(SslMethod::Sslv23).unwrap();
        match ssl_context.set_certificate_file(&*self.cert.clone(), PEM) {
            Ok(_) => info!(target:"Server", "Certificate set"),
            Err(_) => error!(target:"Server", "Can't set certificate file"),
        };
        ssl_context.set_verify(SSL_VERIFY_NONE, None);
        match ssl_context.set_private_key_file(&*self.key.clone(), PEM) {
            Ok(_) => info!(target:"Server", "Private key set"),
            Err(_) => error!(target:"Server", "Can't set private key"),
        };
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let ssl_stream = SslStream::accept(&ssl_context, stream.try_clone().unwrap());
                    let ssl_ok = match ssl_stream {
                        Ok(_) => true,
                        Err(_) => false,
                    };
                    if ssl_ok {
                        let ssl_stream = ssl_stream.unwrap();
                        let client = Client::new(ssl_stream.try_clone().unwrap());
                        self.handle_client(client);
                    } else {
                        error!(target:"Server", "Can't create SslStream");
                    }
                }
                Err(e) => {
                    error!(target:"server", "Connection failed because {}", e);
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
            info!(target:"server", "\n{}", data_received);
            let data_to_process = RoriData::from_json(String::from(data_received));
            if data_to_process.datatype == "register" {
                ENDPOINTMANAGER.lock().unwrap().register_endpoint(data_to_process);
            } else {
                let module_manager = ModuleManager::new(data_to_process);
                module_manager.process();
            }
        });
    }
}


#[derive(Clone, RustcDecodable, RustcEncodable, Default, PartialEq, Debug)]
struct ConfigAPI {
    pub api_ip: Option<String>,
    pub api_port: Option<String>,
}

pub struct API {
    address: String,
}

impl API {
    fn parse_config(data: String) -> String {
        let params: ConfigAPI = decode(&data[..]).unwrap();

        format!("{}:{}",
                &params.api_ip.unwrap_or(String::from("")),
                &params.api_port.unwrap_or(String::from("")))
    }

    pub fn new<P: AsRef<Path>>(config: P) -> API {
        // Configure from file
        let mut file = File::open(config)
            .ok()
            .expect("Config file not found");
        let mut data = String::new();
        file.read_to_string(&mut data)
            .ok()
            .expect("failed to read!");
        let address = API::parse_config(data);
        API { address: address }
    }

    pub fn start(&mut self) {
        let mut router = Router::new();
        router.get("/help", API::help, "help");
        router.get("/rm/:id", API::remove_client, "rm");
        router.get("/client/:owner/:datatype", API::get_client, "client");
        router.post("/send/:id", API::send_from_rori, "send");
        Iron::new(router).http(&*self.address).unwrap();
    }

    #[allow(unused_variables)]
    pub fn help(request: &mut Request) -> IronResult<Response> {
        let help = "RORI API:
        GET rm/:id => remove an endpoint
        GET client/:owner/:datatype => get endpoint list
        POST RoriData to send/:id => send data for client (not implemented)";
        Ok(Response::with((status::Ok, help)))
    }

    pub fn remove_client(request: &mut Request) -> IronResult<Response> {
        let id = request.extensions.get::<Router>().unwrap().find("id").unwrap_or("");
        let id = id.parse::<i64>().unwrap_or(-1);
        if id >= 0 {
            if ENDPOINTMANAGER.lock().unwrap().remove_endpoint(id as u64) {
                return Ok(Response::with((status::Ok, "endpoint removed")));
            }
        }
        Ok(Response::with((status::Ok, "can't remove this endpoint")))
    }

    pub fn get_client(request: &mut Request) -> IronResult<Response> {
        let owner = request.extensions.get::<Router>().unwrap().find("owner").unwrap_or("");
        if owner == "" {
            return Ok(Response::with((status::Ok, "No owner specified")));
        }
        let datatype = request.extensions.get::<Router>().unwrap().find("datatype").unwrap_or("");
        if datatype == "" {
            return Ok(Response::with((status::Ok, "No owner specified")));
        }
        let collected_endpoints = ENDPOINTMANAGER.lock()
            .unwrap()
            .get_endpoint_for_type(String::from(datatype), String::from(owner));

        let encoded_result = json::encode(&collected_endpoints).unwrap();
        Ok(Response::with((status::Ok, encoded_result.to_string())))
    }

    pub fn send_from_rori(request: &mut Request) -> IronResult<Response> {
        let id = request.extensions.get::<Router>().unwrap().find("id").unwrap_or("");
        let id = id.parse::<i64>().unwrap_or(-1);
        if id < 0 {
            return Ok(Response::with((status::Ok, "No id specified")));
        }
        let mut payload = String::from("");
        let _ = request.body.read_to_string(&mut payload);
        ENDPOINTMANAGER.lock()
            .unwrap()
            .send_to_endpoint(id as u64, &payload);
        Ok(Response::with((status::Ok, payload)))
    }
}
