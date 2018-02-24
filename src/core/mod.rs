
pub mod endpoint_manager;
pub mod module_manager;
pub mod words_manager;

use crypto::digest::Digest;
use crypto::sha2::Sha256;
use core::module_manager::ModuleManager;
use core::endpoint_manager::EndpointManager;
use core::words_manager::WordsManager;
use iron::prelude::*;
use iron::status;
use openssl::ssl::{Ssl, SslContext, SslMethod, SslStream, SslVerifyMode, SslFiletype};
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
    pub static ref ENDPOINTMANAGER: Mutex<EndpointManager> = Mutex::new(EndpointManager::new());
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
struct AuthorizedUser {
    pub name: Option<String>,
    pub secret: Option<String>,
}

#[derive(Clone, RustcDecodable, RustcEncodable, Default, PartialEq, Debug)]
struct ConfigServer {
    pub ip: Option<String>,
    pub port: Option<String>,
    pub cert: Option<String>,
    pub key: Option<String>,
    pub authorize: Vec<AuthorizedUser>,
}

pub struct Server {
    address: String,
    cert: String,
    key: String,
    authorize: Vec<AuthorizedUser>,
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
            authorize: params.authorize,
        }
    }

    pub fn start(&mut self) {
        let listener = TcpListener::bind(&*self.address).unwrap();
        let mut ssl_context = SslContext::builder(SslMethod::tls()).unwrap();
        match ssl_context.set_certificate_file(&*self.cert.clone(), SslFiletype::PEM) {
            Ok(_) => info!(target:"Server", "Certificate set"),
            Err(_) => error!(target:"Server", "Can't set certificate file"),
        };
        ssl_context.set_verify(SslVerifyMode::NONE);
        match ssl_context.set_private_key_file(&*self.key.clone(), SslFiletype::PEM) {
            Ok(_) => info!(target:"Server", "Private key set"),
            Err(_) => error!(target:"Server", "Can't set private key"),
        };
        let ssl = ssl_context.build();
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let ssl_stream = Ssl::new(&ssl).unwrap().accept(stream);
                    let ssl_ok = match ssl_stream {
                        Ok(_) => true,
                        Err(_) => false,
                    };
                    if ssl_ok {
                        let ssl_stream = ssl_stream.unwrap();
                        let client = Client::new(ssl_stream);
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

    fn is_authorized(authorize: Vec<AuthorizedUser>, data: RoriData) -> bool {
        let mut hasher = Sha256::new();
        hasher.input_str(&*data.secret);
        let secret = hasher.result_str();
        for client in authorize {
            if client.name.unwrap().to_lowercase() == data.client.to_lowercase() &&
               secret.to_lowercase() == client.secret.unwrap().to_lowercase() {
                return true;
            }
        }
        false
    }

    fn handle_client(&mut self, mut client: Client) {
        let authorized_clients = self.authorize.clone();
        thread::spawn(move || {
            let data_received = client.read();
            let end = data_received.find(0u8 as char);
            let (data_received, _) = data_received.split_at(end.unwrap_or(data_received.len()));
            info!(target:"server", "\n{}", data_received);
            let data_to_process = RoriData::from_json(String::from(data_received));
            let data_authorized = Server::is_authorized(authorized_clients,
                                                        data_to_process.clone());
            if data_authorized {
                if data_to_process.datatype == "register" {
                    ENDPOINTMANAGER.lock().unwrap().register_endpoint(data_to_process);
                } else {
                    let module_manager = ModuleManager::new(data_to_process);
                    module_manager.process();
                }
            } else {
                error!(target:"Server", "Stream not authorized! Don't process.");
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
        router.post("/reprocess", API::reprocess, "reprocess");
        router.get("/add_word/:category/:word",
                   API::add_word_to_category,
                   "add_word");
        router.get("/rm_word/:category/:word",
                   API::remove_word_from_category,
                   "rm_word");
        router.get("/is/:category/:word", API::is_word_in_category, "is_in");
        Iron::new(router).http(&*self.address).unwrap();
    }

    #[allow(unused_variables)]
    pub fn help(request: &mut Request) -> IronResult<Response> {
        let help = "RORI API:
        GET rm/:id => remove an endpoint
        GET client/:owner/:datatype => get endpoint list
        POST RoriData to send/:id => send data for client
        POST RoriData to reprocess/ => reprocess this data and call modules
        GET add_word/:category/:word => Add word to a category
        GET rm_word/:category/:word => Remove word from category
        GET is/:category/:word => Test if a word is in a category";
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
        let mut owner = request.extensions.get::<Router>().unwrap().find("owner").unwrap_or("");
        if owner == "" {
            return Ok(Response::with((status::Ok, "No owner specified")));
        }
        if owner == "*" {
            owner = "";
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

    pub fn reprocess(request: &mut Request) -> IronResult<Response> {
        // Warning, will process and after return OK!
        let mut payload = String::from("");
        let _ = request.body.read_to_string(&mut payload);
        let module_manager = ModuleManager::new(RoriData::from_json(String::from(payload)));
        module_manager.process();
        Ok(Response::with((status::Ok, "")))
    }

    pub fn add_word_to_category(request: &mut Request) -> IronResult<Response> {
        let category = request.extensions.get::<Router>().unwrap().find("category").unwrap_or("");
        let word = request.extensions.get::<Router>().unwrap().find("word").unwrap_or("");
        let mut wm = WordsManager::new(String::from("wordsclassification"));
        wm.add_word_to_category(String::from(word), String::from(category));
        Ok(Response::with((status::Ok, "")))
    }

    pub fn remove_word_from_category(request: &mut Request) -> IronResult<Response> {
        let category = request.extensions.get::<Router>().unwrap().find("category").unwrap_or("");
        let word = request.extensions.get::<Router>().unwrap().find("word").unwrap_or("");
        let mut wm = WordsManager::new(String::from("wordsclassification"));
        wm.remove_word_from_category(String::from(word), String::from(category));
        Ok(Response::with((status::Ok, "")))
    }

    pub fn is_word_in_category(request: &mut Request) -> IronResult<Response> {
        let category = request.extensions.get::<Router>().unwrap().find("category").unwrap_or("");
        let word = request.extensions.get::<Router>().unwrap().find("word").unwrap_or("");
        let wm = WordsManager::new(String::from("wordsclassification"));
        if wm.is_word_in_category(String::from(word), String::from(category)) {
            return Ok(Response::with((status::Ok, "1")));
        }
        Ok(Response::with((status::Ok, "0")))
    }
}
