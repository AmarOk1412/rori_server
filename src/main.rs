extern crate env_logger;
extern crate iron;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate rustc_serialize;
extern crate regex;
extern crate router;

mod core;
mod rori_utils;

use core::Server;
use core::API;
use std::thread;

fn main() {
    // Init logging
    env_logger::init().unwrap();

    // Launch API
    thread::spawn(move || {
        let mut api = API::new("config_server.json");
        api.start();
    });
    // Launch RORI server
    let mut server = Server::new("config_server.json");
    server.start();
}
