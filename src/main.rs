extern crate cpython;
extern crate crypto;
extern crate env_logger;
extern crate iron;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate openssl;
extern crate petgraph;
extern crate rustc_serialize;
extern crate regex;
extern crate router;

pub mod core;
pub mod rori_utils;

use core::Server;
use core::API;
use core::words_manager::WordsManager;
use std::thread;

#[allow(dead_code)]
fn main() {
    let mut wm = WordsManager {
        graph: WordsManager::build_graph(String::from("")),
        path: String::from(""),
    };

    wm.add_word_to_category(String::from("child0"), String::from("parent"));
    wm.add_word_to_category(String::from("child1"), String::from("parent"));
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
