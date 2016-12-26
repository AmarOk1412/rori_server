extern crate rustc_serialize;
extern crate regex;
#[macro_use]
extern crate lazy_static;
extern crate iron;
extern crate router;

mod core;
mod rori_utils;

use core::Server;
use core::API;
use std::thread;

fn main() {
    // TODO API:
    // get client list
    // remove client
    // send message to RORI
    thread::spawn(move || {
        let mut api = API::new("config_server.json");
        api.start();
    });
    let mut server = Server::new("config_server.json");
    server.start();
}
