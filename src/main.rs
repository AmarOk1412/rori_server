extern crate rustc_serialize;
extern crate regex;

mod core;
mod rori_utils;

use core::Server;

fn main() {
    let mut server = Server::new("config_server.json");
    server.start();
}
